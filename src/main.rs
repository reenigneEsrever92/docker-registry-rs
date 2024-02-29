use axum::routing::{delete, get, head, patch, post, put};
use axum::Router;
use tracing::Level;
use crate::db::FilesystemDB;
use clap::Parser;

mod blob;
mod index;
mod db;
mod manifest;
mod model;

#[derive(Default, Clone)]
struct DockerRegistryRS {
    db: FilesystemDB
}

#[derive(Parser)]
struct App {
    #[arg(short, long)]
    port: u16,
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let args = App::parse();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let docker_registry_rs = DockerRegistryRS::default();

    let app = Router::new()
        .route("/v2", get(index::get))
        .route("/v2/", get(index::get))
        .route("/v2/:name/blobs/:digest", head(blob::head))
        .route("/v2/:name/blobs/uploads", post(blob::post))
        .route("/v2/:name/blobs/uploads/", post(blob::post))
        .route("/v2/:name/blobs/uploads/:id", put(blob::put))
        .route("/v2/:name/blobs/uploads/:id", delete(blob::delete))
        .route("/v2/:name/blobs/uploads/:id", patch(blob::patch))
        .route("/v2/:name/manifests/:reference", head(manifest::head))
        .route("/v2/:name/manifests/:reference", put(manifest::put))
        .with_state(docker_registry_rs);
        // .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;

    Ok(axum::serve(listener, app).await?)
}
