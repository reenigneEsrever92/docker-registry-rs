use crate::DockerRegistryRS;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

use dkregistry::reference::ReferenceParseError;

use crate::api::ApiError;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("Reference is invalid")]
    InvalidReference(#[from] ReferenceParseError),
}

pub async fn head(
    State(state): State<DockerRegistryRS>,
    Path((name, reference)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    info!(?name, ?reference, "HEAD manifest");

    let digest = state.db.get_manifest(&name, &reference).await?;

    match digest {
        None => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()),
        Some((size, digest, manifest)) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Docker-Content-Digest", digest)
            .header(header::CONTENT_LENGTH, size)
            .header(header::CONTENT_TYPE, manifest.media_type)
            .body(Body::empty())
            .unwrap()),
    }
}

pub async fn put(
    State(state): State<DockerRegistryRS>,
    Path((name, reference)): Path<(String, String)>,
    body: String,
) -> impl IntoResponse {
    info!(?name, ?reference, "PUT manifest");

    let digest = state
        .db
        .create_manifest(&name, &reference, &body)
        .await
        .unwrap();

    let _repo_ref = state
        .db
        .put_reference(&name, &reference, &digest)
        .await
        .unwrap();

    let url = format!("/v2/{name}/blobs/{digest}");

    Response::builder()
        .status(StatusCode::CREATED)
        .header(header::LOCATION, url)
        .header("Docker-Content-Digest", digest)
        .body(Body::empty())
        .unwrap()
}
