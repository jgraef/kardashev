pub mod admin;

use axum::{
    extract::State,
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing,
    Json,
    Router,
};
use kardashev_protocol::ServerStatus;

use crate::{
    error::Error,
    server::Context,
};

pub fn router() -> Router<Context> {
    Router::new()
        .route("/status", routing::get(get_status))
        .nest("/admin", admin::router())
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

async fn get_status(State(context): State<Context>) -> Json<ServerStatus> {
    Json(ServerStatus {
        server_version: semver_macro::env_version!("CARGO_PKG_VERSION"),
        up_since: context.up_since,
    })
}
