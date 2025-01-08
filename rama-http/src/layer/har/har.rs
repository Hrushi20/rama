use rama_core::{Layer, Service};
use std::future::Future;
use tokio::io::AsyncWrite;
use tokio::sync::mpsc;

// pub struct HarLayer<W: AsyncWrite> {
//     writer: W,
// }

// impl<W: AsyncWrite> HarLayer<W> {
//     pub fn new(writer: W) -> Self {
//         Self { writer }
//     }
// }

// impl<S, W> Layer<S> for HarLayer<W>
// where
//     W: AsyncWrite
// {
//     type Service = HarService<S, W>;
//     fn layer(self, inner: S) -> Self::Service {
//         HarService::new(inner, self.writer)
//     }
// }

struct HarService<S, W:AsyncWrite> {
    inner: S,
    writer: W,          // Async Writer
    toggle_tx: mpsc::Sender<()>,
    recorder: Option<Box<dyn Recorder>>
}

impl<S, W> HarService<S, W> where W: AsyncWrite {
    fn new(inner: S, writer: W) -> Self {
        let (toggle_tx, mut rc) = mpsc::channel::<()>(1);

        tokio::spawn(async move {
            loop {
                let mut active = false;
                tokio::select! {
                    _ = rc.toggle() => { active = !active }
                };

                if active {
                    // Set a Recorder
                    println!("Active");
                } else {
                    println!("Inactive");
                    // disable
                }

            }
        });

        Self {
            writer,
            inner,
            toggle_tx,
            recorder: None
        }
    }

    async fn toggle(&self) {
        self.toggle_tx.send(()).await;
    }
}

// impl<State,S, W, ReqBody, ResBody> Service<State, Request<ReqBody>> for HarService<S, W>{
//     type Response = Response<ResBody>;
//     type Error = BoxError;
//
//     fn serve(&self, ctx: Context<State>, req: Request<ReqBody>) -> impl Future<Output=Result<Self::Response, Self::Error>> + Send + '_ {
//         match self.recorder() {
//
//         }
//     }
// }

trait Toggle {
    fn toggle(&mut self) -> impl Future + Send + '_;
}

impl Toggle for mpsc::Receiver<()> {
    async fn toggle(&mut self) -> () {
        self.recv().await;
    }
}

trait Recorder {
   fn record_request(&self);
   fn record_response(&self);
}


#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test(flavor = "multi_thread")]
    async fn lol(){

        let (tx, mut rc) = mpsc::channel::<()>(1);
        println!("Test started");

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = rc.toggle() => { println!("Executed") }
                };

                println!("I am here executing");
            }
        });

        tx.send(()).await.unwrap();

        sleep(Duration::new(5, 0)).await;


    }


}
