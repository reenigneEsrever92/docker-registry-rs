use crate::api::ApiError;

use crate::DockerRegistryRS;

use axum::extract::{Path, State};


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
