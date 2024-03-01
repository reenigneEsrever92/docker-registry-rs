use crate::model::ManifestV2Schema2;
use crate::DockerRegistryRS;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use dkregistry::reference::ReferenceParseError;
use dkregistry::v2::manifest::ManifestSchema2Spec;
use serde_json::from_str;
use std::str::FromStr;
use thiserror::Error;
use tracing::info;
use tracing_subscriber::fmt::format;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("Reference is invalid")]
    InvalidReference(#[from] ReferenceParseError),
}

pub async fn head(
    State(_state): State<DockerRegistryRS>,
    Path((_name, _reference)): Path<(String, String)>,
) -> impl IntoResponse {
    todo!()
}

pub async fn put(
    State(state): State<DockerRegistryRS>,
    Path((name, reference)): Path<(String, String)>,
    body: String,
) -> impl IntoResponse {
    info!("PUT manifest");

    let digest = state
        .db
        .create_manifest(&name, &reference, &body)
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
