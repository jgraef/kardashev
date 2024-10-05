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
use kardashev_protocol::{
    model::star::{
        CatalogIds,
        Star,
        StarId,
    },
    GetStarsResponse,
    ServerStatus,
};

use crate::{
    context::Context,
    error::Error,
    util::sqlx::{
        Rgb,
        Vec3,
    },
};

pub fn router() -> Router<Context> {
    Router::new()
        .route("/status", routing::get(get_status))
        .nest("/admin", admin::router())
        .route("/star", routing::get(get_stars))
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        tracing::error!(error = ?self, "Internal server error");
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

async fn get_status(State(context): State<Context>) -> Json<ServerStatus> {
    Json(ServerStatus {
        server_version: semver_macro::env_version!("CARGO_PKG_VERSION"),
        up_since: context.up_since,
    })
}

async fn get_stars(State(context): State<Context>) -> Result<Json<GetStarsResponse>, Error> {
    let mut tx = context.transaction().await?;

    let stars = sqlx::query!(
        r#"
        SELECT
            id,
            position AS "position: Vec3",
            effective_temperature,
            color AS "color: Rgb",
            absolute_magnitude,
            luminousity,
            radius,
            mass,
            spectral_type,
            name,
            id_hyg,
            id_hip,
            id_hd,
            id_hr,
            id_gl,
            id_bf
        FROM star
        "#,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|row| {
        Star {
            id: StarId(row.id),
            position: row.position.into(),
            effective_temperature: row.effective_temperature,
            color: row.color.into(),
            absolute_magnitude: row.absolute_magnitude,
            luminousity: row.luminousity,
            radius: row.radius,
            mass: row.mass,
            spectral_type: row.spectral_type,
            name: row.name,
            catalog_ids: CatalogIds {
                hyg: row.id_hyg.map(|id| id as u32),
                hip: row.id_hip.map(|id| id as u32),
                hd: row.id_hd.map(|id| id as u32),
                hr: row.id_hr.map(|id| id as u32),
                gl: row.id_gl,
                bf: row.id_bf,
            },
        }
    })
    .collect();

    Ok(Json(GetStarsResponse { stars }))
}
