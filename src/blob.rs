use crate::DockerRegistryRS;
use axum::body::{Body, HttpBody};
use axum::extract::{Path, Query, Request, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use futures::StreamExt;
use tracing::{debug, info};

pub async fn post(
    State(state): State<DockerRegistryRS>,
    Path(name): Path<String>,
    request: Request,
) -> impl IntoResponse {
    debug!(?request, "Post blob");

    let id = state.db.create_upload(&name).await.unwrap();

    let url = format!("/v2/{name}/blobs/uploads/{id}");

    Response::builder()
        .status(StatusCode::ACCEPTED)
        .header(header::LOCATION, url)
        .header(header::RANGE, "0-0")
        .body(Body::empty())
        .unwrap()
}

pub async fn put(
    State(state): State<DockerRegistryRS>,
    Path((name, id)): Path<(String, String)>,
    Query(digest): Query<String>,
) -> impl IntoResponse {
    state.db.commit_upload(name, id, digest).await.unwrap();

    (StatusCode::ACCEPTED, [(header::RANGE)])
}

pub async fn delete(
    State(state): State<DockerRegistryRS>,
    Path((name, id)): Path<(String, String)>,
) -> impl IntoResponse {
    state.db.delete_upload(&name, &id).await.unwrap();
    StatusCode::NO_CONTENT
}

pub async fn patch(
    State(state): State<DockerRegistryRS>,
    Path((name, id)): Path<(String, String)>,
    request: Request,
) -> impl IntoResponse {
    debug!(?request, "Patch blob");

    let body = request.into_body();

    let (start, end) = state.db.write_upload(&name, &id, body.into_data_stream())
        .await
        .unwrap();

    (
        StatusCode::ACCEPTED,
        [(header::RANGE, format!("{start}-{end}"))],
    )
}
