use crate::infrastructure::storage::traits::{ObjectMetadata, Storage};
use crate::shared::{AppError, AppResult};
use std::path::PathBuf;

pub struct LocalStorage {
    pub root_dir: PathBuf,
}

impl LocalStorage {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }
}

impl Storage for LocalStorage {
    async fn put(&self, _key: &str, _data: Vec<u8>) -> AppResult<String> {
        Err(AppError::Internal("storage not implemented".into()))
    }

    async fn get(&self, _key: &str) -> AppResult<Vec<u8>> {
        Err(AppError::Internal("storage not implemented".into()))
    }

    async fn delete(&self, _key: &str) -> AppResult<()> {
        Err(AppError::Internal("storage not implemented".into()))
    }

    async fn exists(&self, _key: &str) -> AppResult<bool> {
        Err(AppError::Internal("storage not implemented".into()))
    }

    async fn metadata(&self, _key: &str) -> AppResult<ObjectMetadata> {
        Err(AppError::Internal("storage not implemented".into()))
    }
}
