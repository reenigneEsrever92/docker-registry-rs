use crate::DockerRegistryRS;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use dkregistry::reference::ReferenceParseError;
use std::str::FromStr;
use axum::body::Body;
use axum::Json;
use dkregistry::v2::manifest::ManifestSchema2Spec;
use thiserror::Error;

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
    Json(manifest): Json<ManifestSchema2Spec>
) -> impl IntoResponse {
    let reference = dkregistry::reference::Reference::from_str(&reference).unwrap();
    state.db.create_manifest(&name, &reference, &manifest).await;
}
