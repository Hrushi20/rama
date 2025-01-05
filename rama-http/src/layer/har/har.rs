use std::future::Future;
use rama_core::{Context, Layer, Service};
use tokio::io::AsyncWrite;
use rama_core::error::BoxError;
use rama_http_types::{Request, Response};

pub struct HarLayer<W: AsyncWrite> {
    writer: W,
}

impl<W: AsyncWrite> HarLayer<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<S, W> Layer<S> for HarLayer<W>
where
    W: AsyncWrite,
{
    type Service = HarService<S, W>;
    fn layer(self, inner: S) -> Self::Service {
        HarService::new(inner, self.writer)
    }
}

pub struct HarService<S, W:AsyncWrite> {
    is_recording: bool, // Based on if different threads access, convert to Atomic Bool.
    inner: S,
    writer: W,          // Async Writer
}

impl<W, S> HarService<W, S> where W: AsyncWrite {
    fn new(writer: W, inner: S) -> Self {
        Self {
            is_recording: false, // Initially no recording.
            writer,
            inner,
        }
    }
}

impl<State,S, W, ReqBody, ResBody> Service<State, Request<ReqBody>> for HarService<S, W>{
    type Response = Response<ResBody>;
    type Error = BoxError;

    fn serve(&self, ctx: Context<State>, req: Request<ReqBody>) -> impl Future<Output=Result<Self::Response, Self::Error>> + Send + '_ {
        todo!()
    }
}