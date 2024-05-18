use crate::error::{BoxError, ErrorExt, OpaqueError};
use crate::http::client::{ClientConnection, EstablishedClientConnection};
use crate::http::headers::{Authorization, ProxyAuthorization};
use crate::http::{Request, RequestContext};
use crate::proxy::{ProxyCredentials, ProxySocketAddr};
use crate::service::{Context, Layer, Service};
use crate::stream::Stream;
use std::fmt;
use std::future::Future;
use std::net::SocketAddr;

use super::HttpProxyConnector;

/// A [`Layer`] which wraps the given service with a [`HttpProxyConnectorService`].
///
/// See [`HttpProxyConnectorService`] for more information.
pub struct HttpProxyConnectorLayer<P> {
    provider: P,
}

impl<P: std::fmt::Debug> std::fmt::Debug for HttpProxyConnectorLayer<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpProxyConnectorLayer")
            .field("provider", &self.provider)
            .finish()
    }
}

impl<P: Clone> Clone for HttpProxyConnectorLayer<P> {
    fn clone(&self) -> Self {
        Self {
            provider: self.provider.clone(),
        }
    }
}

#[derive(Debug, Clone)]
/// Minimal information required to establish a connection over an HTTP Proxy.
pub struct HttpProxyInfo {
    /// The proxy address to connect to.
    pub proxy: SocketAddr,
    /// The credentials to use for the proxy connection.
    pub credentials: Option<ProxyCredentials>,
}

// TOOD: support from ENV + ENV DEFAULT (HTTP_PROXY)

impl HttpProxyConnectorLayer<HttpProxyInfo> {
    /// Creates a new [`HttpProxyConnectorLayer`].
    pub fn proxy_static(info: HttpProxyInfo) -> Self {
        Self { provider: info }
    }
}

impl HttpProxyConnectorLayer<private::FromContext> {
    /// Creates a new [`HttpProxyConnectorLayer`] which will establish
    /// a proxy connection in case the context contains a [`HttpProxyInfo`].
    pub fn proxy_from_context() -> Self {
        Self {
            provider: private::FromContext,
        }
    }
}

impl<S, P: Clone> Layer<S> for HttpProxyConnectorLayer<P> {
    type Service = HttpProxyConnectorService<S, P>;

    fn layer(&self, inner: S) -> Self::Service {
        HttpProxyConnectorService::new(self.provider.clone(), inner)
    }
}

/// A connector which can be used to establish a connection over an HTTP Proxy.
pub struct HttpProxyConnectorService<S, P> {
    inner: S,
    provider: P,
}

impl<S: fmt::Debug, P: fmt::Debug> fmt::Debug for HttpProxyConnectorService<S, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpProxyConnectorService")
            .field("inner", &self.inner)
            .field("provider", &self.provider)
            .finish()
    }
}

impl<S: Clone, P: Clone> Clone for HttpProxyConnectorService<S, P> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            provider: self.provider.clone(),
        }
    }
}

impl<S, P> HttpProxyConnectorService<S, P> {
    /// Creates a new [`HttpProxyConnectorService`].
    pub fn new(provider: P, inner: S) -> Self {
        Self { inner, provider }
    }
}

impl<S> HttpProxyConnectorService<S, HttpProxyInfo> {
    /// Creates a new [`HttpProxyConnectorService`] which will establish
    /// a proxied connection over the given proxy info.
    pub fn proxy_static(info: HttpProxyInfo, inner: S) -> Self {
        Self::new(info, inner)
    }
}

impl<S> HttpProxyConnectorService<S, private::FromContext> {
    /// Creates a new [`HttpProxyConnectorService`] which will establish
    /// a proxied connection if the context contains the info,
    /// otherwise it will establish a direct connection.
    pub fn proxy_from_context(inner: S) -> Self {
        Self::new(private::FromContext, inner)
    }
}

impl<S, State, Body, T, P> Service<State, Request<Body>> for HttpProxyConnectorService<S, P>
where
    S: Service<State, Request<Body>, Response = EstablishedClientConnection<T, Body, State>>,
    T: Stream + Unpin,
    P: HttpProxyProvider<State>,
    P::Error: Into<BoxError>,
    S::Error: Into<BoxError>,
    State: Send + Sync + 'static,
    Body: Send + 'static,
{
    type Response = EstablishedClientConnection<T, Body, State>;
    type Error = OpaqueError;

    async fn serve(
        &self,
        ctx: Context<State>,
        req: Request<Body>,
    ) -> Result<Self::Response, Self::Error> {
        let private::HttpProxyOutput { info, mut ctx } =
            self.provider.info(ctx).await.map_err(|err| {
                OpaqueError::from_boxed(err.into()).context("fetch proxy info from provider")
            })?;

        // in case the provider gave us a proxy info, we insert it into the context
        if let Some(info) = info.as_ref() {
            ctx.insert(ProxySocketAddr::new(info.proxy));
        }

        let established_conn =
            self.inner.serve(ctx, req).await.map_err(|err| {
                OpaqueError::from_boxed(err.into()).context("establish inner stream")
            })?;

        // return early in case we did not use a proxy
        let info = match info {
            Some(info) => info,
            None => {
                return Ok(established_conn);
            }
        };
        // and do the handshake otherwise...

        let EstablishedClientConnection { mut ctx, req, conn } = established_conn;

        let (addr, stream) = conn.into_parts();

        let request_context = ctx.get_or_insert_with(|| RequestContext::new(&req));
        let authority = match request_context.authority() {
            Some(authority) => authority,
            None => {
                return Err(OpaqueError::from_display("missing http authority"));
            }
        };

        let mut connector = HttpProxyConnector::new(authority);
        if let Some(credentials) = info.credentials.as_ref() {
            match credentials {
                ProxyCredentials::Basic { username, password } => {
                    let c = Authorization::basic(
                        username.as_str(),
                        password.as_deref().unwrap_or_default(),
                    )
                    .0;
                    connector.with_typed_header(ProxyAuthorization(c));
                }
                ProxyCredentials::Bearer(token) => {
                    let c = Authorization::bearer(token.as_str())
                        .map_err(|err| {
                            OpaqueError::from_std(err).context("define http proxy bearer token")
                        })?
                        .0;
                    connector.with_typed_header(ProxyAuthorization(c));
                }
            }
        }

        let stream = connector
            .handshake(stream)
            .await
            .map_err(|err| OpaqueError::from_std(err).context("http proxy handshake"))?;

        Ok(EstablishedClientConnection {
            ctx,
            req,
            conn: ClientConnection::new(addr, stream),
        })
    }
}

pub trait HttpProxyProvider<S>: private::Sealed<S> {}

impl<S, T> HttpProxyProvider<S> for T where T: private::Sealed<S> {}

mod private {
    use std::{convert::Infallible, sync::Arc};

    use super::*;

    #[derive(Debug)]
    pub struct HttpProxyOutput<S> {
        pub info: Option<HttpProxyInfo>,
        pub ctx: Context<S>,
    }

    #[derive(Debug, Clone)]
    pub struct FromContext;

    pub trait Sealed<S>: Clone + Send + Sync + 'static {
        type Error;

        fn info(
            &self,
            ctx: Context<S>,
        ) -> impl Future<Output = Result<HttpProxyOutput<S>, Self::Error>> + Send + '_;
    }

    impl<S, T> Sealed<S> for Arc<T>
    where
        T: Sealed<S>,
    {
        type Error = T::Error;

        fn info(
            &self,
            ctx: Context<S>,
        ) -> impl Future<Output = Result<HttpProxyOutput<S>, Self::Error>> + Send + '_ {
            (**self).info(ctx)
        }
    }

    impl<S> Sealed<S> for HttpProxyInfo
    where
        S: Send + Sync + 'static,
    {
        type Error = Infallible;

        async fn info(&self, ctx: Context<S>) -> Result<HttpProxyOutput<S>, Self::Error> {
            Ok(HttpProxyOutput {
                info: Some(self.clone()),
                ctx,
            })
        }
    }

    impl<S> Sealed<S> for FromContext
    where
        S: Send + Sync + 'static,
    {
        type Error = Infallible;

        async fn info(&self, ctx: Context<S>) -> Result<HttpProxyOutput<S>, Self::Error> {
            let info = ctx.get::<HttpProxyInfo>().cloned();
            Ok(HttpProxyOutput { info, ctx })
        }
    }
}