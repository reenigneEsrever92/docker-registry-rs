use axum::body::Bytes;
use axum::Error;
use futures::{Stream, StreamExt};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::io;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tracing::{debug, warn};
use uuid::Uuid;

pub const REGISTRY_PATH: &'static str = ".local/share/registry-rs";
pub const REPOSITORY_FOLDER: &'static str = "repositories";
pub const BLOB_PATH: &'static str = "blobs";

#[derive(Debug, Error)]
pub enum UploadError {
    #[error("Filesystem Error")]
    FilesystemError(#[from] io::Error),
    #[error("Upload does not exist")]
    UploadNotExists { hash: String },
    #[error("Axum error")]
    AxumError(#[from] axum::Error),
}

pub type UploadResult<T> = Result<T, UploadError>;

#[derive(Debug, Clone)]
pub struct FilesystemDBConfig {
    base_path: String,
}

impl Default for FilesystemDBConfig {
    fn default() -> Self {
        Self {
            base_path: ".local/share/registry-rs".to_string(),
        }
    }
}

#[derive(Default, Clone)]
pub struct FilesystemDB {
    config: FilesystemDBConfig,
}

impl FilesystemDB {

    #[allow(unused)]
    fn new(config: FilesystemDBConfig) -> Self {
        Self { config }
    }

    pub async fn create_upload(&self, name: &str) -> UploadResult<Uuid> {
        let path = format!("{}/{REPOSITORY_FOLDER}/{name}/_uploads", self.config.base_path);

        tokio::fs::create_dir_all(&path).await?;

        let id = Uuid::new_v4();

        let _file = tokio::fs::File::create(format!("{path}/{id}")).await?;

        Ok(id)
    }

    pub async fn delete_upload(&self, name: &str, id: &str) -> UploadResult<()> {
        tokio::fs::remove_file(format!("{}/{name}/uploads/{id}", self.config.base_path)).await?;
        Ok(())
    }

    pub async fn commit_upload(&self, name: &str, id: &str, digest: &str) -> UploadResult<()> {
        let upload_path = format!("{}/{name}/uploads/{id}", self.config.base_path);
        let blob_path = tokio::fs::remove_file(upload_path).await?;

        Ok(())
    }

    pub async fn write_upload<D>(&self, name: &str, id: &str, mut data: D) -> UploadResult<(usize, usize)>
        where
            D: Stream<Item = Result<Bytes, Error>> + Unpin,
    {
        let mut file = tokio::fs::File::open(format!("{REGISTRY_PATH}/{name}/uploads/{id}")).await?;

        let bytes_in_file = file.metadata().await?.len() as usize;
        let mut bytes_written = 0;

        while let Some(bytes) = data.next().await {
            debug!(?bytes, "Received bytes");

            let bytes = bytes?;
            bytes_written += bytes.len();

            file.write_all(&bytes).await?;

            debug!(?file, ?bytes_in_file, "Written bytes")
        }

        Ok((bytes_in_file, bytes_in_file + bytes_written))
    }
}




