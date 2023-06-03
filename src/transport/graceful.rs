//! graceful shutdown utilities

use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use pin_project_lite::pin_project;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_util::sync::{CancellationToken, WaitForCancellationFuture};

pin_project! {
    pub struct ShutdownFuture<'a> {
        #[pin]
        maybe_future: Option<WaitForCancellationFuture<'a>>,
    }
}

impl<'a> ShutdownFuture<'a> {
    pub fn new(future: WaitForCancellationFuture<'a>) -> Self {
        Self {
            maybe_future: Some(future),
        }
    }

    pub fn pending() -> Self {
        Self { maybe_future: None }
    }
}

impl Future for ShutdownFuture<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().maybe_future.as_pin_mut() {
            Some(fut) => fut.poll(cx),
            None => Poll::Pending,
        }
    }
}

/// A service to facilitate graceful shutdown within your server.
pub struct GracefulService {
    shutdown: CancellationToken,
    shutdown_complete_rx: Receiver<()>,
    shutdown_complete_tx: Sender<()>,
}

/// Create the service required to facilitate graceful shutdown within your server.
pub fn service(signal: impl Future + Send + 'static) -> GracefulService {
    GracefulService::new(signal)
}

/// The error returned in case a graceful service that was blocked on shutdown
/// using a deadline (duration) that was reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeoutError(());

impl Display for TimeoutError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Graceful shutdown timed out")
    }
}

impl Error for TimeoutError {}

impl GracefulService {
    pub fn new(signal: impl Future + Send + 'static) -> Self {
        let shutdown = CancellationToken::new();
        let (shutdown_complete_tx, shutdown_complete_rx) = channel(1);

        let token = shutdown.clone();
        tokio::spawn(async move {
            let _ = signal.await;
            token.cancel();
        });

        Self {
            shutdown,
            shutdown_complete_rx,
            shutdown_complete_tx,
        }
    }

    /// Create a new graceful token that can be used by a graceful service's
    /// child processes to indicate it is finished as well as to interrupt itself
    /// in case a shutdown is desired.
    pub fn token(&self) -> Token {
        Token::new(
            self.shutdown.child_token(),
            self.shutdown_complete_tx.clone(),
        )
    }

    /// Wait indefinitely until the server has its shutdown requested
    pub async fn shutdown_req(&self) {
        self.shutdown.cancelled().await;
    }

    /// Wait indefinitely until the server can be gracefully shut down.
    pub async fn shutdown(mut self) {
        self.shutdown.cancelled().await;
        drop(self.shutdown_complete_tx);
        self.shutdown_complete_rx.recv().await;
    }

    /// Wait until the server is gracefully shutdown,
    /// but adding a max amount of time to wait since the moment
    /// a cancellation it desired.
    pub async fn shutdown_until(mut self, duration: Duration) -> Result<(), TimeoutError> {
        self.shutdown.cancelled().await;
        drop(self.shutdown_complete_tx);
        match tokio::time::timeout(duration, self.shutdown_complete_rx.recv()).await {
            Err(_) => Err(TimeoutError(())),
            Ok(_) => Ok(()),
        }
    }
}

impl Default for GracefulService {
    fn default() -> Self {
        let signal = tokio::signal::ctrl_c();
        Self::new(signal)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    state: Option<TokenState>,
}

impl Token {
    // Construct a true graceful token.
    //
    // This token will drop the shutdown_complete
    // when finished (to mark it went out of scope) and which can be also used
    // to await the given shutdown cancellation token.
    pub fn new(shutdown: CancellationToken, shutdown_complete: Sender<()>) -> Self {
        Self {
            state: Some(TokenState {
                shutdown,
                shutdown_complete,
            }),
        }
    }

    // Construct a token that will never shutdown.
    //
    // This is a desired solution where you need to provide a token for
    // a service which is not graceful.
    pub fn pending() -> Self {
        Self { state: None }
    }

    pub fn shutdown(&self) -> ShutdownFuture<'_> {
        match &self.state {
            Some(state) => ShutdownFuture::new(state.shutdown.cancelled()),
            None => ShutdownFuture::pending(),
        }
    }

    pub fn child_token(&self) -> Token {
        match &self.state {
            Some(state) => Token {
                state: Some(TokenState {
                    shutdown: state.shutdown.child_token(),
                    shutdown_complete: state.shutdown_complete.clone(),
                }),
            },
            None => self.clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct TokenState {
    shutdown: CancellationToken,
    shutdown_complete: Sender<()>,
}

#[cfg(test)]
mod tests {
    use std::future::pending;

    use super::*;

    use tokio::{select, time::sleep};

    #[tokio::test]
    async fn test_token_pending() {
        let token = Token::pending();
        select! {
            _ = token.shutdown() => panic!("should not shutdown"),
            _ = sleep(Duration::from_millis(100)) => (),
        };
    }

    #[tokio::test]
    async fn test_graceful_service() {
        let (tx, mut rx) = channel::<()>(1);
        let (shutdown_tx, mut shutdown_rx) = channel::<()>(1);

        let service_shutdown_tx = shutdown_tx.clone();
        let service = service(async move {
            let _ = rx.recv().await.unwrap();
            drop(service_shutdown_tx);
        });

        let token = service.token();
        let process_shutdown_tx = shutdown_tx.clone();
        tokio::spawn(async move {
            token.shutdown().await;
            drop(process_shutdown_tx);
        });

        tokio::spawn(async move {
            sleep(Duration::from_millis(100)).await;
            tx.send(()).await.unwrap();
        });

        service.shutdown().await;

        drop(shutdown_tx);
        shutdown_rx.recv().await;
    }

    #[tokio::test]
    async fn test_graceful_service_timeout() {
        let (tx, mut rx) = channel::<()>(1);
        let (shutdown_tx, mut shutdown_rx) = channel::<()>(1);

        let service_shutdown_tx = shutdown_tx.clone();
        let service = service(async move {
            let _ = rx.recv().await.unwrap();
            drop(service_shutdown_tx);
        });

        let token = service.token();
        tokio::spawn(async move {
            pending::<()>().await;
            token.shutdown().await;
        });

        tokio::spawn(async move {
            sleep(Duration::from_millis(100)).await;
            tx.send(()).await.unwrap();
        });

        assert_eq!(
            TimeoutError(()),
            service
                .shutdown_until(Duration::from_millis(100))
                .await
                .unwrap_err(),
        );

        drop(shutdown_tx);
        shutdown_rx.recv().await;
    }
}
