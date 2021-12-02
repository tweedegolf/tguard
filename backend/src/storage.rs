use crate::Error;
use cloud_storage::Object;
use rocket::tokio::fs::File;
use rocket::tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::path::Path;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn store(&self, data: Vec<u8>, file_name: &str) -> Result<(), Error>;
    async fn retrieve_url(&self, file_name: &str) -> Result<String, Error>;
    async fn serve(&self, slug: &str) -> Result<Vec<u8>, Error>;
}

pub struct LocalStorage {
    pub host: String,
    pub directory: String,
}

#[async_trait]
impl Storage for LocalStorage {
    async fn store(&self, data: Vec<u8>, file_name: &str) -> Result<(), Error> {
        let mut file = File::create(Path::new(&self.directory).join(file_name)).await?;
        file.write_all(&data).await?;

        Ok(())
    }

    async fn retrieve_url(&self, file_name: &str) -> Result<String, Error> {
        Ok(format!("{}/api/storage/{}", self.host, file_name))
    }

    async fn serve(&self, file_name: &str) -> Result<Vec<u8>, Error> {
        let mut file = File::open(Path::new(&self.directory).join(file_name)).await?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).await?;

        return Ok(data);
    }
}

pub struct CloudStorage {
    pub bucket: String,
}

#[async_trait]
impl Storage for CloudStorage {
    async fn store(&self, data: Vec<u8>, file_name: &str) -> Result<(), Error> {
        Object::create(&self.bucket, data, file_name, "application/octet-stream").await?;

        Ok(())
    }

    async fn retrieve_url(&self, file_name: &str) -> Result<String, Error> {
        let object = Object::read(&self.bucket, file_name).await?;
        Ok(object.download_url(30)?)
    }

    async fn serve(&self, _: &str) -> Result<Vec<u8>, Error> {
        Err(Error::NotFound)
    }
}
