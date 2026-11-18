use crate::shared::AppResult;

#[derive(Debug, Clone)]
pub struct ObjectMetadata {
    pub size: u64,
    pub content_type: Option<String>,
}

#[allow(async_fn_in_trait)]
pub trait Storage {
    async fn put(&self, key: &str, data: Vec<u8>) -> AppResult<String>;

    async fn get(&self, key: &str) -> AppResult<Vec<u8>>;

    async fn delete(&self, key: &str) -> AppResult<()>;

    async fn exists(&self, key: &str) -> AppResult<bool>;

    async fn metadata(&self, key: &str) -> AppResult<ObjectMetadata>;
}
