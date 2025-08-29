//! S3-compatible storage backend

use super::*;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{
    config::{Credentials, SharedAsyncRead, SharedAsyncSeek, SharedAsyncWrite},
    operation::{
        create_bucket::CreateBucketOutput, delete_object::DeleteObjectOutput,
        get_object::GetObjectOutput, list_objects_v2::ListObjectsV2Output,
        put_object::PutObjectOutput,
    },
    primitives::{ByteStream, SdkBody},
    types::{BucketLocationConstraint, CreateBucketConfiguration},
    Client,
};
use aws_smithy_http::byte_stream::ByteStream as SmithyByteStream;
use std::path::Path;
use tokio::io::AsyncWriteExt;

/// S3 storage backend
#[derive(Debug, Clone)]
pub struct S3Storage {
    client: Client,
    bucket: String,
}

impl S3Storage {
    /// Create a new S3 storage backend
    pub async fn new(
        bucket: &str,
        region: &str,
        endpoint: Option<&str>,
        access_key_id: &str,
        secret_access_key: &str,
    ) -> Result<Self> {
        let region_provider = RegionProviderChain::first_try(region.parse().map(Some)?);
        
        let mut s3_config = aws_sdk_s3::config::Builder::new()
            .region(region_provider)
            .credentials_provider(Credentials::new(
                access_key_id,
                secret_access_key,
                None,
                None,
                "rastos-backup",
            ));

        // Use custom endpoint if provided (for MinIO, etc.)
        if let Some(endpoint) = endpoint {
            s3_config = s3_config.endpoint_url(endpoint);
        }

        let client = Client::from_conf(s3_config.build());

        // Ensure the bucket exists
        if let Err(e) = client.head_bucket().bucket(bucket).send().await {
            if e.is_no_such_bucket() {
                Self::create_bucket(&client, bucket, region).await?;
            } else {
                return Err(BackupError::Storage(e.into()));
            }
        }

        Ok(Self {
            client,
            bucket: bucket.to_string(),
        })
    }

    async fn create_bucket(client: &Client, bucket: &str, region: &str) -> Result<()> {
        let constraint = BucketLocationConstraint::from(region);
        let cfg = CreateBucketConfiguration::builder()
            .location_constraint(constraint)
            .build();

        client
            .create_bucket()
            .bucket(bucket)
            .create_bucket_configuration(cfg)
            .send()
            .await
            .map_err(|e| BackupError::Storage(e.into()))?;

        Ok(())
    }

    fn normalize_path(&self, path: &Path) -> String {
        // Convert path to forward slashes for S3
        path.to_string_lossy().replace('\\', "/")
    }
}

#[async_trait]
impl StorageBackend for S3Storage {
    async fn put(&self, path: &Path, data: bytes::Bytes) -> Result<()> {
        let key = self.normalize_path(path);
        let body = ByteStream::from(data);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body)
            .send()
            .await
            .map_err(|e| BackupError::Storage(e.into()))?;

        Ok(())
    }

    async fn get(&self, path: &Path) -> Result<bytes::Bytes> {
        let key = self.normalize_path(path);

        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| BackupError::Storage(e.into()))?;

        let data = response
            .body
            .collect()
            .await
            .map_err(|e| BackupError::Storage(e.into()))?;

        Ok(data.into_bytes())
    }

    async fn list(
        &self,
        prefix: Option<&Path>,
    ) -> Result<Vec<object_store::path::Path>> {
        let prefix = prefix.map(|p| self.normalize_path(p));

        let mut response = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .set_prefix(prefix.clone())
            .into_paginator()
            .send();

        let mut paths = Vec::new();

        while let Some(result) = response.next().await {
            let output = result.map_err(|e| BackupError::Storage(e.into()))?;

            for object in output.contents() {
                if let Some(key) = object.key() {
                    if let Ok(path) = object_store::path::Path::parse(key) {
                        paths.push(path);
                    }
                }
            }
        }

        Ok(paths)
    }

    async fn delete(&self, path: &Path) -> Result<()> {
        let key = self.normalize_path(path);

        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| BackupError::Storage(e.into()))?;

        Ok(())
    }

    async fn exists(&self, path: &Path) -> bool {
        let key = self.normalize_path(path);

        self.client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .is_ok()
    }
}
