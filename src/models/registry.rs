<<<<<<< HEAD
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModelRegistry {
    registry_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryModel {
    pub name: String,
    pub tags: Vec<String>,
    pub description: String,
    pub downloads: HashMap<String, ModelDownloadInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelDownloadInfo {
    pub url: String,
    pub size: u64,
    pub sha256: String,
    pub format: String,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            registry_url: "https://registry.ollama.ai".to_string(),
        }
    }
    
    pub async fn get_model_info(&self, model_name: &str) -> Result<RegistryModel> {
        let parts: Vec<&str> = model_name.split(':').collect();
        let name = parts[0];
        let tag = parts.get(1).unwrap_or(&"latest");
        
        let url = match name.to_lowercase().as_str() {
            name if name.contains("llama") => {
                format!("https://huggingface.co/meta-llama/{}/resolve/main/model.gguf", name)
            },
            name if name.contains("qwen") => {
                format!("https://huggingface.co/Qwen/{}/resolve/main/model.gguf", name)
            },
            name if name.contains("gemma") => {
                format!("https://huggingface.co/google/{}/resolve/main/model.gguf", name)
            },
            _ => {
                format!("{}/v2/library/{}/manifests/{}", self.registry_url, name, tag)
            }
        };
        
        Ok(RegistryModel {
            name: name.to_string(),
            tags: vec![tag.to_string()],
            description: format!("{} model", name),
            downloads: HashMap::from([
                (tag.to_string(), ModelDownloadInfo {
                    url,
                    size: 0,
                    sha256: String::new(),
                    format: "gguf".to_string(),
                })
            ]),
        })
    }
    
    pub fn get_huggingface_url(&self, model_name: &str) -> Result<String> {
        let parts: Vec<&str> = model_name.split(':').collect();
        let name = parts[0];
        let name_lower = name.to_lowercase();
        let tag = parts.get(1).unwrap_or(&"latest");
        
        let url = if name.contains('/') {
            // Direct HuggingFace path: user/repo-name
            // Try to find a GGUF file - common patterns
            let repo = name;
            if name_lower.contains("gguf") {
                // Repo name contains GGUF, likely has GGUF files
                format!("https://huggingface.co/{}/resolve/main/model.gguf", repo)
            } else {
                format!("https://huggingface.co/{}-GGUF/resolve/main/model.gguf", repo)
            }
        } else if name_lower.contains("llama4") || name_lower.contains("llama-4") || name_lower.contains("llama3") {
            "https://huggingface.co/QuantFactory/Meta-Llama-3.1-8B-Instruct-GGUF/resolve/main/Meta-Llama-3.1-8B-Instruct.Q4_K_M.gguf".to_string()
        } else if name_lower.contains("qwen3") || name_lower.contains("qwen-3") || name_lower.contains("qwen2") {
            "https://huggingface.co/Qwen/Qwen2.5-7B-Instruct-GGUF/resolve/main/qwen2.5-7b-instruct-q4_k_m.gguf".to_string()
        } else if name_lower.contains("gemma3") || name_lower.contains("gemma-3") || name_lower.contains("gemma2") {
            if tag.contains("27b") {
                "https://huggingface.co/bartowski/gemma-2-27b-it-GGUF/resolve/main/gemma-2-27b-it-Q4_K_M.gguf".to_string()
            } else {
                "https://huggingface.co/bartowski/gemma-2-9b-it-GGUF/resolve/main/gemma-2-9b-it-Q4_K_M.gguf".to_string()
            }
        } else if name_lower.contains("mistral") {
            "https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q4_K_M.gguf".to_string()
        } else if name_lower.contains("phi") {
            "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf".to_string()
        } else {
            return Err(anyhow!("Unknown model: {}. Try using a direct HuggingFace path like 'username/model-GGUF'", model_name));
        };
        
        Ok(url)
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
=======
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModelRegistry {
    registry_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryModel {
    pub name: String,
    pub tags: Vec<String>,
    pub description: String,
    pub downloads: HashMap<String, ModelDownloadInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelDownloadInfo {
    pub url: String,
    pub size: u64,
    pub sha256: String,
    pub format: String,
}

#[derive(Debug, Deserialize)]
struct HfFileInfo {
    path: String,
    size: Option<u64>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            registry_url: "https://huggingface.co".to_string(),
        }
    }
    
    pub async fn get_model_info(&self, model_name: &str) -> Result<RegistryModel> {
        let parts: Vec<&str> = model_name.split(':').collect();
        let name = parts[0];
        let tag = parts.get(1).unwrap_or(&"latest");
        
        let url = self.get_huggingface_url(model_name)?;
        
        Ok(RegistryModel {
            name: name.to_string(),
            tags: vec![tag.to_string()],
            description: format!("{} model", name),
            downloads: HashMap::from([
                (tag.to_string(), ModelDownloadInfo {
                    url,
                    size: 0,
                    sha256: String::new(),
                    format: "gguf".to_string(),
                })
            ]),
        })
    }
    
    /// Discover GGUF files from a HuggingFace repository
    pub async fn discover_gguf_files(&self, repo: &str) -> Result<Vec<(String, String, u64)>> {
        let api_url = format!("https://huggingface.co/api/models/{}/tree/main", repo);
        
        let client = reqwest::Client::new();
        let response = client.get(&api_url)
            .header("User-Agent", "rust-llm-runner")
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch repo info: HTTP {}", response.status()));
        }
        
        let files: Vec<HfFileInfo> = response.json().await?;
        
        let gguf_files: Vec<(String, String, u64)> = files.into_iter()
            .filter(|f| f.path.to_lowercase().ends_with(".gguf"))
            .map(|f| {
                let url = format!("{}/{}/resolve/main/{}", self.registry_url, repo, f.path);
                (f.path, url, f.size.unwrap_or(0))
            })
            .collect();
        
        Ok(gguf_files)
    }
    
    /// Get the best GGUF file URL for a model (prefers Q4_K_M quantization)
    pub async fn get_best_gguf_url(&self, repo: &str) -> Result<(String, String, u64)> {
        let files = self.discover_gguf_files(repo).await?;
        
        if files.is_empty() {
            return Err(anyhow!("No GGUF files found in repository: {}", repo));
        }
        
        // Priority order for quantization levels
        let priorities = ["q4_k_m", "q4_k_s", "q5_k_m", "q5_k_s", "q8_0", "q6_k", "f16"];
        
        for priority in priorities {
            if let Some(file) = files.iter().find(|(name, _, _)| name.to_lowercase().contains(priority)) {
                return Ok(file.clone());
            }
        }
        
        // Return first available GGUF
        Ok(files.into_iter().next().unwrap())
    }
    
    pub fn get_huggingface_url(&self, model_name: &str) -> Result<String> {
        let parts: Vec<&str> = model_name.split(':').collect();
        let name = parts[0];
        let name_lower = name.to_lowercase();
        let tag = parts.get(1).unwrap_or(&"latest");
        
        // Check if it's a local file path
        if std::path::Path::new(name).exists() && name.to_lowercase().ends_with(".gguf") {
            return Ok(name.to_string());
        }
        
        let url = if name.contains('/') {
            // Direct HuggingFace path: user/repo-name
            let repo = name;
            if name_lower.contains("gguf") {
                // Repo already has GGUF - will be discovered dynamically
                format!("hf://{}", repo)
            } else {
                // Try with -GGUF suffix
                format!("hf://{}-GGUF", repo)
            }
        } else if name_lower.contains("llama4") || name_lower.contains("llama-4") {
            "https://huggingface.co/unsloth/Llama-4-Scout-17B-16E-Instruct-GGUF/resolve/main/Llama-4-Scout-17B-16E-Instruct-UD-Q4_K_XL.gguf".to_string()
        } else if name_lower.contains("llama3") || name_lower.contains("llama-3") {
            "https://huggingface.co/QuantFactory/Meta-Llama-3.1-8B-Instruct-GGUF/resolve/main/Meta-Llama-3.1-8B-Instruct.Q4_K_M.gguf".to_string()
        } else if name_lower.contains("qwen3") || name_lower.contains("qwen-3") {
            "https://huggingface.co/Qwen/Qwen3-8B-GGUF/resolve/main/qwen3-8b-q4_k_m.gguf".to_string()
        } else if name_lower.contains("qwen2") || name_lower.contains("qwen-2") {
            "https://huggingface.co/Qwen/Qwen2.5-7B-Instruct-GGUF/resolve/main/qwen2.5-7b-instruct-q4_k_m.gguf".to_string()
        } else if name_lower.contains("gemma3") || name_lower.contains("gemma-3") {
            if tag.contains("27b") {
                "https://huggingface.co/bartowski/google_gemma-3-27b-it-GGUF/resolve/main/google_gemma-3-27b-it-Q4_K_M.gguf".to_string()
            } else {
                "https://huggingface.co/bartowski/gemma-3-12b-it-GGUF/resolve/main/gemma-3-12b-it-Q4_K_M.gguf".to_string()
            }
        } else if name_lower.contains("gemma2") || name_lower.contains("gemma-2") {
            if tag.contains("27b") {
                "https://huggingface.co/bartowski/gemma-2-27b-it-GGUF/resolve/main/gemma-2-27b-it-Q4_K_M.gguf".to_string()
            } else {
                "https://huggingface.co/bartowski/gemma-2-9b-it-GGUF/resolve/main/gemma-2-9b-it-Q4_K_M.gguf".to_string()
            }
        } else if name_lower.contains("mistral") {
            "https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q4_K_M.gguf".to_string()
        } else if name_lower.contains("phi") {
            "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf".to_string()
        } else if name_lower.contains("deepseek") {
            "https://huggingface.co/bartowski/DeepSeek-R1-Distill-Qwen-7B-GGUF/resolve/main/DeepSeek-R1-Distill-Qwen-7B-Q4_K_M.gguf".to_string()
        } else {
            return Err(anyhow!(
                "Unknown model: {}. Supported shortcuts: llama3, llama4, qwen2, qwen3, gemma2, gemma3, mistral, phi, deepseek\n\
                Or use HuggingFace path: 'owner/repo-GGUF' (e.g., 'bartowski/gemma-3-27b-it-GGUF')", 
                model_name
            ));
        };
        
        Ok(url)
    }
    
    /// Resolve a model URL - handles both direct URLs and hf:// prefixed dynamic discovery
    pub async fn resolve_model_url(&self, model_name: &str) -> Result<String> {
        let url = self.get_huggingface_url(model_name)?;
        
        if url.starts_with("hf://") {
            // Dynamic discovery needed
            let repo = url.strip_prefix("hf://").unwrap();
            let (filename, resolved_url, size) = self.get_best_gguf_url(repo).await?;
            println!("Discovered GGUF: {} ({:.2} GB)", filename, size as f64 / 1024.0 / 1024.0 / 1024.0);
            Ok(resolved_url)
        } else {
            Ok(url)
        }
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
>>>>>>> bb9577f (20260204_220651)
