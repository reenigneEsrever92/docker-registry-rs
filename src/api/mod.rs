use crate::db::DBError;
use crate::DockerRegistryRS;
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, head, patch, post, put};
use axum::Router;
use serde::Serialize;
use thiserror::Error;

mod blob;
mod manifest;
mod repository;

pub(crate) fn get_api() -> Router {
    let docker_registry_rs = DockerRegistryRS::default();

    Router::new()
        .route("/v2", get(index))
        .route("/v2/", get(index))
        .route("/v2/:name/blobs/:digest", head(blob::head))
        .route("/v2/:name/blobs/uploads", post(blob::post))
        .route("/v2/:name/blobs/uploads/", post(blob::post))
        .route("/v2/:name/blobs/uploads/:id", put(blob::put))
        .route("/v2/:name/blobs/uploads/:id", delete(blob::delete))
        .route("/v2/:name/blobs/uploads/:id", patch(blob::patch))
        .route("/v2/:name/manifests/:reference", head(manifest::head))
        .route("/v2/:name/manifests/:reference", put(manifest::put))
        .route("/v2/:name/tags/list", get(repository::list))
        .with_state(docker_registry_rs)
}

pub async fn index() -> impl IntoResponse {
    StatusCode::OK
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap(),
            ApiError::DatabaseError(_) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap(),
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum ApiError {
    #[error("Resource not found")]
    NotFound,
    #[error("Database error")]
    DatabaseError(#[from] DBError),
}

#[derive(Debug, Serialize)]
pub(crate) struct ErrorResponse {
    errors: Vec<ErrorMessage>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ErrorMessage {
    code: String,
    message: String,
    detail: String,
}
