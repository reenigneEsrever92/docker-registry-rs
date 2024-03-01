use crate::DockerRegistryRS;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use dkregistry::reference::ReferenceParseError;
use std::str::FromStr;
use axum::body::Body;
use axum::http::header;
use axum::Json;
use dkregistry::v2::manifest::ManifestSchema2Spec;
use serde_json::from_str;
use thiserror::Error;
use tracing::info;
use tracing_subscriber::fmt::format;
use crate::model::ManifestV2Schema2;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("Reference is invalid")]
    InvalidReference(#[from] ReferenceParseError),
}

pub async fn head(Path((name, reference)): Path<(String, String)>) -> impl IntoResponse {
    todo!()
}

pub async fn put(
    State(state): State<DockerRegistryRS>,
    Path((name, reference)): Path<(String, String)>,
    body: String
) -> impl IntoResponse {
    info!("PUT manifest");

    let digest = state.db.create_manifest(&name, &reference, &body).await.unwrap();

    let url = format!("/v2/{name}/blobs/{digest}");

    Response::builder()
        .header(header::LOCATION, url)
        .body(Body::empty())
        .unwrap()
}
