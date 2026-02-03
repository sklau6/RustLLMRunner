use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use crate::models::metadata::{ModelMetadata, MetadataStore};
use crate::config::Config;
use crate::inference::engine::InferenceEngine;

pub struct ModelManager {
    config: Arc<Config>,
    metadata_store: Arc<MetadataStore>,
    loaded_models: Arc<RwLock<HashMap<String, Arc<InferenceEngine>>>>,
}

impl ModelManager {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let metadata_store = Arc::new(MetadataStore::new(&config.db_path)?);
        let loaded_models = Arc::new(RwLock::new(HashMap::new()));
        
        Ok(Self {
            config,
            metadata_store,
            loaded_models,
        })
    }
    
    pub async fn load_model(&self, name: &str, tag: &str) -> Result<Arc<InferenceEngine>> {
        let key = format!("{}:{}", name, tag);
        
        {
            let models = self.loaded_models.read().await;
            if let Some(engine) = models.get(&key) {
                return Ok(Arc::clone(engine));
            }
        }
        
        let metadata = self.metadata_store.get_model(name, tag)?
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", key))?;
        
        let engine = Arc::new(InferenceEngine::new(&metadata.path, self.config.clone())?);
        
        {
            let mut models = self.loaded_models.write().await;
            
            if models.len() >= self.config.max_loaded_models {
                if let Some(oldest_key) = models.keys().next().cloned() {
                    models.remove(&oldest_key);
                }
            }
            
            models.insert(key.clone(), Arc::clone(&engine));
        }
        
        Ok(engine)
    }
    
    pub async fn unload_model(&self, name: &str, tag: &str) -> Result<()> {
        let key = format!("{}:{}", name, tag);
        let mut models = self.loaded_models.write().await;
        models.remove(&key);
        Ok(())
    }
    
    pub async fn list_loaded_models(&self) -> Vec<String> {
        let models = self.loaded_models.read().await;
        models.keys().cloned().collect()
    }
    
    pub fn save_metadata(&self, metadata: &ModelMetadata) -> Result<()> {
        self.metadata_store.save_model(metadata)
    }
    
    pub fn get_metadata(&self, name: &str, tag: &str) -> Result<Option<ModelMetadata>> {
        self.metadata_store.get_model(name, tag)
    }
    
    pub fn list_all_models(&self) -> Result<Vec<ModelMetadata>> {
        self.metadata_store.list_models()
    }
    
    pub fn delete_metadata(&self, name: &str, tag: &str) -> Result<()> {
        self.metadata_store.delete_model(name, tag)
    }
}
