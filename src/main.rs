use crate::api::get_api;
use crate::db::FilesystemDB;
use clap::Parser;
use tracing::Level;

mod api;
mod db;
mod model;

#[derive(Default, Clone)]
struct DockerRegistryRS {
    db: FilesystemDB,
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

    let app = get_api();

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;

    Ok(axum::serve(listener, app).await?)
}
