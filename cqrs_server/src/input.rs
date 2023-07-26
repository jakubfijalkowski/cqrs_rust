use axum::{
    async_trait,
    body::HttpBody,
    extract::{rejection::JsonRejection, FromRequest},
    http::Request,
    response::{IntoResponse, Response},
    BoxError, Json,
};
use serde::de::DeserializeOwned;

pub struct CQRSInput<T>(pub T);
pub struct CQRSJsonRejection(JsonRejection);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for CQRSInput<T>
where
    T: DeserializeOwned,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = CQRSJsonRejection;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let inner = Json::<T>::from_request(req, state).await;
        inner
            .map(|Json(i)| CQRSInput(i))
            .map_err(|c| CQRSJsonRejection(c))
    }
}

impl IntoResponse for CQRSJsonRejection {
    fn into_response(self) -> Response {
        let mut response = self.0.into_response();
        *response.status_mut() = axum::http::StatusCode::UNPROCESSABLE_ENTITY;
        response
    }
}
