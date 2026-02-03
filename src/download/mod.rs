use anyhow::{Result, Context};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use sha2::{Sha256, Digest};

pub struct Downloader {
    client: Client,
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("rust-llm-runner/0.1.0")
                .build()
                .unwrap(),
        }
    }
    
    pub async fn download_file(&self, url: &str, dest_path: &Path) -> Result<()> {
        tracing::info!("Downloading from: {}", url);
        
        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to download: HTTP {}", response.status());
        }
        
        let total_size = response.content_length().unwrap_or(0);
        
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
                .progress_chars("#>-"),
        );
        
        if let Some(parent) = dest_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        let mut file = File::create(dest_path).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }
        
        pb.finish_with_message("Download complete");
        file.flush().await?;
        
        tracing::info!("Downloaded to: {}", dest_path.display());
        Ok(())
    }
    
    pub async fn verify_checksum(&self, file_path: &Path, expected_hash: &str) -> Result<bool> {
        let mut file = tokio::fs::File::open(file_path).await?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 8192];
        
        loop {
            let n = tokio::io::AsyncReadExt::read(&mut file, &mut buffer).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        
        let hash = format!("{:x}", hasher.finalize());
        Ok(hash == expected_hash)
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}
