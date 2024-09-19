use super::FromRequestParts;
use crate::dep::http::request::Parts;
use crate::utils::macros::define_http_rejection;
use rama_core::Context;
use rama_net::address;
use rama_net::http::RequestContext;
use rama_utils::macros::impl_deref;

/// Extractor that resolves the hostname of the request.
#[derive(Debug, Clone)]
pub struct Host(pub address::Host);

impl_deref!(Host: address::Host);

define_http_rejection! {
    #[status = BAD_REQUEST]
    #[body = "Failed to detect the Http host"]
    /// Rejection type used if the [`Host`] extractor is unable to
    /// determine the (http) Host.
    pub struct MissingHost;
}

impl<S> FromRequestParts<S> for Host
where
    S: Send + Sync + 'static,
{
    type Rejection = MissingHost;

    async fn from_request_parts(ctx: &Context<S>, parts: &Parts) -> Result<Self, Self::Rejection> {
        Ok(Host(match ctx.get::<RequestContext>() {
            Some(ctx) => ctx.authority.host().clone(),
            None => RequestContext::try_from((ctx, parts))
                .map_err(|_| MissingHost)?
                .authority
                .host()
                .clone(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::dep::http_body_util::BodyExt as _;
    use crate::header::X_FORWARDED_HOST;
    use crate::layer::forwarded::GetForwardedHeadersService;
    use crate::service::web::WebService;
    use crate::StatusCode;
    use crate::{Body, HeaderName, Request};
    use rama_core::Service;

    async fn test_host_from_request(uri: &str, host: &str, headers: Vec<(&HeaderName, &str)>) {
        let svc = GetForwardedHeadersService::x_forwarded_host(
            WebService::default().get("/", |Host(host): Host| async move { host.to_string() }),
        );

        let mut builder = Request::builder().method("GET").uri(uri);
        for (header, value) in headers {
            builder = builder.header(header, value);
        }
        let req = builder.body(Body::empty()).unwrap();

        let res = svc.serve(Context::default(), req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body, host);
    }

    #[tokio::test]
    async fn host_header() {
        test_host_from_request(
            "/",
            "some-domain",
            vec![(&http::header::HOST, "some-domain:123")],
        )
        .await;
    }

    #[tokio::test]
    async fn x_forwarded_host_header() {
        test_host_from_request(
            "/",
            "some-domain",
            vec![(&X_FORWARDED_HOST, "some-domain:456")],
        )
        .await;
    }

    #[tokio::test]
    async fn x_forwarded_host_precedence_over_host_header() {
        test_host_from_request(
            "/",
            "some-domain",
            vec![
                (&X_FORWARDED_HOST, "some-domain:456"),
                (&http::header::HOST, "some-domain:123"),
            ],
        )
        .await;
    }

    #[tokio::test]
    async fn uri_host() {
        test_host_from_request("http://example.com", "example.com", vec![]).await;
    }
}