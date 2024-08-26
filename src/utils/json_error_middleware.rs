use async_trait::async_trait;

use poem::{Endpoint, IntoResponse, Middleware, Request, Response};
use poem_openapi::payload::Json;

pub struct JsonErrorMiddleware;

impl<E: Endpoint> Middleware<E> for JsonErrorMiddleware {
    type Output = JsonErrorMiddlewareImpl<E>;

    fn transform(&self, ep: E) -> Self::Output {
        JsonErrorMiddlewareImpl(ep)
    }
}

pub struct JsonErrorMiddlewareImpl<E>(E);

#[async_trait]
impl<E: Endpoint> Endpoint for JsonErrorMiddlewareImpl<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> poem::Result<Self::Output> {
        let res = self.0.call(req).await;

        match res {
            Ok(resp) => Ok(resp.into_response()),
            Err(err) => Ok(Json(
                serde_json::json!({"error": err.to_string(),"isError": true,"statusCode": err.status().as_u16()}),
            )
            .with_status(err.status())
            .into_response()),
        }
    }
}
