use axum::routing::{delete, get, head, patch, post, put};
use axum::Router;
use tower_http::trace::TraceLayer;
use tracing::Level;
use crate::db::FilesystemDB;

mod blob;
mod index;
mod db;
mod manifest;

#[derive(Default, Clone)]
struct DockerRegistryRS {
    db: FilesystemDB
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let docker_registry_rs = DockerRegistryRS::default();

    let app = Router::new()
        .route("/v2", get(index::get))
        .route("/v2/", get(index::get))
        .route("/v2/:name/blobs/uploads", post(blob::post))
        .route("/v2/:name/blobs/uploads/", post(blob::post))
        .route("/v2/:name/blobs/uploads/:id", put(blob::put))
        .route("/v2/:name/blobs/uploads/:id", delete(blob::delete))
        .route("/v2/:name/blobs/uploads/:id", patch(blob::patch))
        .route("/v2/:name/manifests/:reference", head(manifest::head))
        .with_state(docker_registry_rs);
        // .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    let serve = axum::serve(listener, app);
    serve.await.unwrap();
}
