use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub tag: String,
    pub size: u64,
    pub digest: String,
    pub format: String,
    pub family: String,
    pub parameter_size: String,
    pub quantization_level: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub path: String,
}

pub struct MetadataStore {
    db: sled::Db,
}

impl MetadataStore {
    pub fn new(db_path: &Path) -> Result<Self> {
        let db = sled::open(db_path)?;
        Ok(Self { db })
    }
    
    pub fn save_model(&self, metadata: &ModelMetadata) -> Result<()> {
        let key = format!("{}:{}", metadata.name, metadata.tag);
        let value = serde_json::to_vec(metadata)?;
        self.db.insert(key.as_bytes(), value)?;
        self.db.flush()?;
        Ok(())
    }
    
    pub fn get_model(&self, name: &str, tag: &str) -> Result<Option<ModelMetadata>> {
        let key = format!("{}:{}", name, tag);
        if let Some(value) = self.db.get(key.as_bytes())? {
            let metadata: ModelMetadata = serde_json::from_slice(&value)?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }
    
    pub fn list_models(&self) -> Result<Vec<ModelMetadata>> {
        let mut models = Vec::new();
        for item in self.db.iter() {
            let (_, value) = item?;
            let metadata: ModelMetadata = serde_json::from_slice(&value)?;
            models.push(metadata);
        }
        Ok(models)
    }
    
    pub fn delete_model(&self, name: &str, tag: &str) -> Result<()> {
        let key = format!("{}:{}", name, tag);
        self.db.remove(key.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }
}
