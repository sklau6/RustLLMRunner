pub mod registry;
pub mod metadata;
pub mod manager;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub tag: String,
    pub size: u64,
    pub digest: String,
    pub modified_at: DateTime<Utc>,
    pub format: ModelFormat,
    pub family: String,
    pub parameter_size: String,
    pub quantization_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelFormat {
    GGUF,
    SafeTensors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDetails {
    pub parent_model: String,
    pub format: String,
    pub family: String,
    pub families: Vec<String>,
    pub parameter_size: String,
    pub quantization_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManifest {
    pub schema_version: u32,
    pub media_type: String,
    pub config: LayerInfo,
    pub layers: Vec<LayerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerInfo {
    pub media_type: String,
    pub digest: String,
    pub size: u64,
}
