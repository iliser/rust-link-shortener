use poem::{IntoResponse, Response};

use poem_openapi::{
    payload::Payload,
    registry::{MetaResponses, MetaSchemaRef, Registry},
    types::Type,
    ApiResponse,
};

pub struct RawPoemResponse(pub Response);

impl Payload for RawPoemResponse {
    const CONTENT_TYPE: &'static str = "text/plain; charset=utf-8";

    fn schema_ref() -> MetaSchemaRef {
        String::schema_ref()
    }
}

impl IntoResponse for RawPoemResponse {
    fn into_response(self) -> Response {
        self.0
    }
}

impl ApiResponse for RawPoemResponse {
    fn meta() -> MetaResponses {
        MetaResponses { responses: vec![] }
    }

    fn register(_: &mut Registry) {}
}