use crate::api::ApiError;
use crate::db::DBError;
use crate::DockerRegistryRS;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct References {
    name: String,
    tags: Vec<String>,
}

pub(crate) async fn list(
    State(state): State<DockerRegistryRS>,
    Path(name): Path<String>,
) -> Result<Json<References>, ApiError> {
    let tags = state.db.get_references(&name).await.unwrap();

    match tags {
        Some(tags) => Ok(Json(References { name, tags })),
        None => Err(ApiError::NotFound),
    }
}
