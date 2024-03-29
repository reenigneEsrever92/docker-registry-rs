use axum::body::Bytes;
use axum::Error;

use futures::{Stream, StreamExt};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tracing::debug;
use uuid::Uuid;
use crate::model::ManifestV2Schema2;

pub const REGISTRY_PATH: &str = ".local/share/registry-rs";
pub const REPOSITORY_FOLDER: &str = "repositories";
pub const UPLOADS_FOLDER: &str = "_uploads";
pub const REFERENCES_FOLDER: &str = "_refs";
pub const BLOB_FOLDER: &str = "blobs";

#[derive(Debug, Error)]
pub enum DBError {
    #[error("Filesystem Error")]
    FilesystemError(#[from] std::io::Error),
    #[error("Upload does not exist - id: {id}")]
    UploadNotExists { id: String },
    #[error("Upload does not exist - digest: {digest}")]
    BlobNotExists { digest: String },
    #[error("Axum error")]
    AxumError(#[from] axum::Error),
    #[error("Digest is not formatted correctly: {0}")]
    InvalidDigest(String),
    #[error("Content computes to digest: {computed}, and given was: {given}")]
    DigestsDontMatch { given: String, computed: String },
}

pub type DBResult<T> = Result<T, DBError>;

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

impl FilesystemDB {}

impl FilesystemDB {
    #[allow(unused)]
    pub fn new(config: FilesystemDBConfig) -> Self {
        Self { config }
    }

    pub(crate) async fn get_manifest(&self, name: &str, reference: &str) -> DBResult<Option<(u64, String, ManifestV2Schema2)>> {
        let reference_path = self.get_reference_path(name, reference);

        if tokio::fs::try_exists(&reference_path).await? {
            let digest = tokio::fs::read_to_string(&reference_path).await?;
            let (size, path) = self.get_blob(&digest).await?;
            let json = tokio::fs::read_to_string(&path).await?;
            let manifest = serde_json::from_str::<ManifestV2Schema2>(&json).unwrap();

            Ok(Some((size, digest, manifest)))
        } else {
            Ok(None)
        }
    }

    pub(crate) async fn get_references(&self, name: &str) -> DBResult<Option<Vec<String>>> {
        let repository_path = self.get_references_path(name);

        if tokio::fs::try_exists(&repository_path).await? {
            let mut file = tokio::fs::read_dir(&repository_path).await?;
            let mut references = Vec::new();

            while let Some(file) = file.next_entry().await? {
                references.push(file.file_name().into_string().unwrap());
            }

            Ok(Some(references))
        } else {
            Ok(None)
        }
    }

    pub async fn create_manifest(
        &self,
        name: &str,
        reference: &str,
        manifest: &str,
    ) -> DBResult<String> {
        let formatted_digest = format!("sha256:{}", sha256::digest(manifest));
        let path = self.get_blob_path(&formatted_digest)?;

        debug!(
            ?name,
            ?reference,
            ?manifest,
            ?formatted_digest,
            ?path,
            "Creating Manifest"
        );

        tokio::fs::create_dir_all(PathBuf::from(&path).parent().unwrap()).await?;
        tokio::fs::write(path, manifest).await?;

        Ok(formatted_digest)
    }

    pub async fn get_blob(&self, digest: &str) -> DBResult<(u64, String)> {
        let blob_path = self.get_blob_path(digest)?;

        if tokio::fs::try_exists(&blob_path).await? {
            let file = tokio::fs::File::open(&blob_path).await?;

            Ok((file.metadata().await?.len(), blob_path))
        } else {
            Err(DBError::BlobNotExists {
                digest: digest.to_string(),
            })
        }
    }

    pub async fn create_upload(&self, name: &str) -> DBResult<Uuid> {
        let id = Uuid::new_v4();
        let path = self.get_upload_path(name, &id.to_string());

        tokio::fs::create_dir_all(PathBuf::from(&path).parent().unwrap()).await?;

        let _file = tokio::fs::File::create(path).await?;

        Ok(id)
    }

    pub async fn delete_upload(&self, name: &str, id: &str) -> DBResult<()> {
        tokio::fs::remove_file(self.get_upload_path(name, id)).await?;
        Ok(())
    }

    pub async fn commit_upload(&self, name: &str, id: &str, digest: &str) -> DBResult<()> {
        let upload_path = self.get_upload_path(name, id);
        let layers_path = self.get_blob_path(digest)?;

        let local_digest = self.get_digest(&upload_path).await?;

        debug!(
            ?upload_path,
            ?layers_path,
            ?digest,
            ?local_digest,
            "Committing layer"
        );

        if local_digest != self.extract_digest(digest)?.1 {
            tokio::fs::remove_file(&upload_path).await?;

            Err(DBError::DigestsDontMatch {
                given: digest.to_string(),
                computed: local_digest,
            })
        } else {
            tokio::fs::create_dir_all(PathBuf::from(&layers_path).parent().unwrap()).await?;
            tokio::fs::copy(&upload_path, &layers_path).await?;
            tokio::fs::remove_file(&upload_path).await?;

            Ok(())
        }
    }

    pub async fn write_upload<D>(
        &self,
        name: &str,
        upload_id: &str,
        mut data: D,
    ) -> DBResult<(usize, usize)>
    where
        D: Stream<Item = Result<Bytes, Error>> + Unpin,
    {
        let upload_path = self.get_upload_path(name, upload_id);
        let mut file = tokio::fs::File::create(&upload_path).await?;

        let bytes_in_file = file.metadata().await?.len() as usize;
        let mut bytes_written = 0;

        while let Some(bytes) = data.next().await {
            let bytes = bytes?;

            bytes_written += bytes.len();

            file.write_all(&bytes).await?;
        }

        debug!(?name, ?upload_id, ?upload_path, ?bytes_written, "Written bytes");

        file.flush().await?;

        Ok((bytes_in_file, bytes_in_file + bytes_written - 1))
    }

    fn extract_digest(&self, digest: &str) -> DBResult<(String, String)> {
        if let Some((algo, hash)) = digest.split_once(':') {
            Ok((algo.to_string(), hash.to_string()))
        } else {
            Err(DBError::InvalidDigest(digest.to_string()))
        }
    }

    fn get_reference_path(&self, name: &str, reference: &str) -> String {
        format!(
            "{}/{REFERENCES_FOLDER}/{reference}",
            self.get_repository_path(name)
        )
    }

    fn get_references_path(&self, name: &str) -> String {
        format!("{}/{REFERENCES_FOLDER}/", self.get_repository_path(name))
    }

    fn get_repository_path(&self, name: &str) -> String {
        format!("{}/{REPOSITORY_FOLDER}/{name}", self.config.base_path)
    }

    fn get_upload_path(&self, name: &str, id: &str) -> String {
        format!("{}/{UPLOADS_FOLDER}/{id}", self.get_repository_path(name))
    }

    fn get_blob_path(&self, digest: &str) -> DBResult<String> {
        if let Some((algo, hash)) = digest.split_once(':') {
            let bucket = hash.chars().take(2).collect::<String>();
            Ok(format!(
                "{}/{BLOB_FOLDER}/{algo}/{bucket}/{hash}",
                self.config.base_path
            ))
        } else {
            Err(DBError::InvalidDigest(digest.to_string()))
        }
    }

    async fn get_digest(&self, path: &str) -> DBResult<String> {
        let mut hasher = Sha256::new();
        let file = tokio::fs::File::open(path).await?;
        let mut buf = [0; 1024];
        let mut reader = BufReader::new(file);

        while let bytes_read = reader.read(&mut buf).await? {
            if bytes_read == 0 {
                break;
            }
            hasher.update(&mut buf[..bytes_read]);
        }

        unsafe { Ok(format!("{:02x}", hasher.finalize())) }
    }
    pub(crate) async fn put_reference(
        &self,
        name: &str,
        reference: &str,
        digest: &str,
    ) -> DBResult<String> {
        let ref_path = self.get_reference_path(name, reference);

        tokio::fs::create_dir_all(PathBuf::from(&ref_path).parent().unwrap()).await?;
        tokio::fs::write(&ref_path, digest).await?;

        Ok(ref_path)
    }
}
