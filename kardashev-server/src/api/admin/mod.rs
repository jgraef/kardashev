use axum::{
    extract::State,
    routing,
    Json,
    Router,
};
use kardashev_protocol::{
    admin::{
        CreateStarsRequest,
        CreateStarsResponse,
    },
    model::star::StarId,
};

use crate::{
    error::Error,
    server::Context,
    util::sqlx::{
        Rgb,
        Vec3,
    },
};

pub fn router() -> Router<Context> {
    Router::new()
        .route("/star", routing::post(create_stars))
        .route(
            "/shutdown",
            routing::get(|State(context): State<Context>| {
                async move {
                    context.shutdown();
                }
            }),
        )
}

async fn create_stars(
    State(context): State<Context>,
    Json(request): Json<CreateStarsRequest>,
) -> Result<Json<CreateStarsResponse>, Error> {
    let mut tx = context.transaction().await?;

    let mut star_ids = vec![];
    for star in request.stars {
        let row = sqlx::query!(
            r#"
            INSERT INTO star (
                position,
                effective_temperature,
                color,
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
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id
            "#,
            Vec3::from(star.position) as _,
            star.effective_temperature,
            Rgb::from(star.color) as _,
            star.absolute_magnitude,
            star.luminousity,
            star.radius,
            star.mass,
            star.spectral_type,
            star.name,
            star.catalog_ids.hyg.map(|id| id as i32),
            star.catalog_ids.hip.map(|id| id as i32),
            star.catalog_ids.hd.map(|id| id as i32),
            star.catalog_ids.hr.map(|id| id as i32),
            star.catalog_ids.gl,
            star.catalog_ids.bf,
        )
        .fetch_one(&mut **tx)
        .await?;
        star_ids.push(StarId(row.id));
    }

    tx.commit().await?;

    Ok(Json(CreateStarsResponse { ids: star_ids }))
}
