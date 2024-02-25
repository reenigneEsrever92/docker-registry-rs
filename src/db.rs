use axum::body::Bytes;
use axum::Error;
use futures::{Stream, StreamExt};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::io;
use std::path::PathBuf;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tracing::{debug, warn};
use uuid::Uuid;

pub const REGISTRY_PATH: &str = ".local/share/registry-rs";
pub const REPOSITORY_FOLDER: &str = "repositories";
pub const UPLOADS_FOLDER: &str = "_uploads";
pub const LAYERS_FOLDER: &str = "_layers";
pub const BLOB_PATH: &str = "blobs";

#[derive(Debug, Error)]
pub enum UploadError {
    #[error("Filesystem Error")]
    FilesystemError(#[from] io::Error),
    #[error("Upload does not exist")]
    UploadNotExists { hash: String },
    #[error("Axum error")]
    AxumError(#[from] axum::Error),
    #[error("Invalid digest: {0}")]
    InvalidDigest(String)
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
    pub fn new(config: FilesystemDBConfig) -> Self {
        Self { config }
    }

    pub async fn create_upload(&self, name: &str) -> UploadResult<Uuid> {
        let id = Uuid::new_v4();
        let path = self.get_uploads_folder(name, &id.to_string());

        tokio::fs::create_dir_all(PathBuf::from(&path).parent().unwrap()).await?;

        let _file = tokio::fs::File::create(path).await?;

        Ok(id)
    }

    pub async fn delete_upload(&self, name: &str, id: &str) -> UploadResult<()> {
        tokio::fs::remove_file(self.get_uploads_folder(name, id)).await?;
        Ok(())
    }

    pub async fn commit_upload(&self, name: &str, id: &str, digest: &str) -> UploadResult<()> {
        let upload_path = self.get_uploads_folder(name, id);
        let layers_path = self.get_layers_folder(name, digest)?;

        tokio::fs::copy(upload_path, layers_path).await?;

        Ok(())
    }

    pub async fn write_upload<D>(
        &self,
        name: &str,
        id: &str,
        mut data: D,
    ) -> UploadResult<(usize, usize)>
    where
        D: Stream<Item = Result<Bytes, Error>> + Unpin,
    {
        let mut file = tokio::fs::File::open(self.get_uploads_folder(name, id)).await?;

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

    fn get_repository_folder(&self, name: &str) -> String {
        format!("{}/{REPOSITORY_FOLDER}/{name}", self.config.base_path)
    }

    fn get_uploads_folder(&self, name: &str, id: &str) -> String {
        format!("{}/{UPLOADS_FOLDER}/{id}", self.get_repository_folder(name))
    }

    fn get_layers_folder(&self, name: &str, digest: &str) -> UploadResult<String> {
         if let Some((algo, hash)) = digest.split_once('.') {
             Ok(format!("{}/{LAYERS_FOLDER}/{algo}/{hash}", self.get_repository_folder(name)))
         } else {
             Err(UploadError::InvalidDigest(digest.to_string()))
         }
    }
}
