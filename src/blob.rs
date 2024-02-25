use crate::db::UploadResult;
use crate::DockerRegistryRS;
use axum::body::{Body, HttpBody};
use axum::extract::{Path, Query, Request, State};
use axum::http::{header, HeaderMap, HeaderName, StatusCode};
use axum::response::{IntoResponse, Response};
use futures::StreamExt;
use serde::Deserialize;
use sha256::digest;
use std::collections::HashMap;
use tracing::{debug, info};

pub async fn post(
    State(state): State<DockerRegistryRS>,
    Path(name): Path<String>,
    request: Request,
) -> impl IntoResponse {
    info!(?request, "POST blob");

    let id = state.db.create_upload(&name).await.unwrap();

    let url = format!("/v2/{name}/blobs/uploads/{id}");

    Response::builder()
        .status(StatusCode::ACCEPTED)
        .header(header::LOCATION, url)
        .header(header::RANGE, "0-0")
        .body(Body::empty())
        .unwrap()
}

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    digest: String,
}

pub async fn put(
    State(state): State<DockerRegistryRS>,
    Path((name, id)): Path<(String, String)>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    info!(?name, ?id, ?params, "PUT blob");

    state
        .db
        .commit_upload(&name, &id, &params.digest)
        .await
        .unwrap();

    let digest = params.digest;
    let url = format!("/v2/{name}/blobs/{digest}");

    (
        StatusCode::ACCEPTED,
        [
            (header::LOCATION, url),
            (HeaderName::from_static("Docker-Content-Digest"), digest),
        ],
    )
}

pub async fn delete(
    State(state): State<DockerRegistryRS>,
    Path((name, id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(?name, ?id, "DELETE blob");

    state.db.delete_upload(&name, &id).await.unwrap();
    StatusCode::NO_CONTENT
}

pub async fn patch(
    State(state): State<DockerRegistryRS>,
    Path((name, id)): Path<(String, String)>,
    request: Request,
) -> impl IntoResponse {
    info!(?request, "PATCH blob");

    let body = request.into_body();

    let (start, end) = state
        .db
        .write_upload(&name, &id, body.into_data_stream())
        .await
        .unwrap();

    (
        StatusCode::ACCEPTED,
        [(header::RANGE, format!("{start}-{end}"))],
    )
}

pub async fn head(
    State(state): State<DockerRegistryRS>,
    Path((_name, digest)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(?digest, "HEAD blob");

    match state.db.get_blob(&digest).await {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .header("Docker-Content-Digest", digest)
            .body(Body::empty())
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
    }
}
