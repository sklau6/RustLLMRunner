use anyhow::Result;
use std::sync::Arc;
use chrono::Utc;
use dialoguer::{Input, theme::ColorfulTheme};

use crate::config::Config;
use crate::models::manager::ModelManager;
use crate::models::registry::ModelRegistry;
use crate::models::metadata::ModelMetadata;
use crate::download::Downloader;
use crate::inference::{GenerationConfig, GenerationRequest};

pub async fn pull_model(model_name: &str) -> Result<()> {
    println!("Pulling model: {}", model_name);
    
    let config = Arc::new(Config::load()?);
    let registry = ModelRegistry::new();
    let downloader = Downloader::new();
    let model_manager = ModelManager::new(config.clone())?;
    
    let model_parts: Vec<&str> = model_name.split(':').collect();
    let name = model_parts[0];
    let tag = model_parts.get(1).unwrap_or(&"latest");
    
    let url = registry.get_huggingface_url(model_name)?;
    
    let model_path = config.get_model_path(&format!("{}_{}.gguf", name, tag));
    
    println!("Downloading from: {}", url);
    downloader.download_file(&url, &model_path).await?;
    
    let file_size = tokio::fs::metadata(&model_path).await?.len();
    
    let metadata = ModelMetadata {
        name: name.to_string(),
        tag: tag.to_string(),
        size: file_size,
        digest: format!("sha256:{}", uuid::Uuid::new_v4()),
        format: "gguf".to_string(),
        family: name.to_string(),
        parameter_size: tag.to_string(),
        quantization_level: "Q4_K_M".to_string(),
        created_at: Utc::now(),
        modified_at: Utc::now(),
        path: model_path.to_string_lossy().to_string(),
    };
    
    model_manager.save_metadata(&metadata)?;
    
    println!("✓ Successfully pulled {}", model_name);
    
    Ok(())
}

pub async fn list_models() -> Result<()> {
    let config = Arc::new(Config::load()?);
    let model_manager = ModelManager::new(config)?;
    
    let models = model_manager.list_all_models()?;
    
    if models.is_empty() {
        println!("No models found. Use 'pull' to download a model.");
        return Ok(());
    }
    
    println!("\nAvailable models:");
    println!("{:-<80}", "");
    println!("{:<30} {:<15} {:<20} {:<15}", "NAME", "SIZE", "MODIFIED", "FORMAT");
    println!("{:-<80}", "");
    
    for model in models {
        let size_mb = model.size as f64 / 1024.0 / 1024.0;
        let modified = model.modified_at.format("%Y-%m-%d %H:%M:%S");
        println!(
            "{:<30} {:<15} {:<20} {:<15}",
            format!("{}:{}", model.name, model.tag),
            format!("{:.2} MB", size_mb),
            modified,
            model.format
        );
    }
    
    println!("{:-<80}", "");
    
    Ok(())
}

pub async fn run_model(model_name: &str, prompt: Option<String>) -> Result<()> {
    let config = Arc::new(Config::load()?);
    let model_manager = ModelManager::new(config.clone())?;
    
    let model_parts: Vec<&str> = model_name.split(':').collect();
    let name = model_parts[0];
    let tag = model_parts.get(1).unwrap_or(&"latest");
    
    println!("Loading model: {}...", model_name);
    let engine = model_manager.load_model(name, tag).await?;
    
    if let Some(p) = prompt {
        let response = engine.generate(GenerationRequest {
            prompt: p,
            config: GenerationConfig::default(),
            context: None,
        }).await?;
        
        println!("\n{}", response.text);
    } else {
        println!("\nInteractive mode. Type 'exit' to quit.\n");
        
        loop {
            let input: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt(">>>")
                .interact_text()?;
            
            if input.trim().eq_ignore_ascii_case("exit") {
                break;
            }
            
            let response = engine.generate(GenerationRequest {
                prompt: input,
                config: GenerationConfig::default(),
                context: None,
            }).await?;
            
            println!("\n{}\n", response.text);
        }
    }
    
    Ok(())
}

pub async fn remove_model(model_name: &str) -> Result<()> {
    let config = Arc::new(Config::load()?);
    let model_manager = ModelManager::new(config.clone())?;
    
    let model_parts: Vec<&str> = model_name.split(':').collect();
    let name = model_parts[0];
    let tag = model_parts.get(1).unwrap_or(&"latest");
    
    if let Some(metadata) = model_manager.get_metadata(name, tag)? {
        let model_path = std::path::Path::new(&metadata.path);
        if model_path.exists() {
            tokio::fs::remove_file(model_path).await?;
        }
        
        model_manager.delete_metadata(name, tag)?;
        println!("✓ Removed model: {}", model_name);
    } else {
        println!("Model not found: {}", model_name);
    }
    
    Ok(())
}

pub async fn show_model(model_name: &str) -> Result<()> {
    let config = Arc::new(Config::load()?);
    let model_manager = ModelManager::new(config)?;
    
    let model_parts: Vec<&str> = model_name.split(':').collect();
    let name = model_parts[0];
    let tag = model_parts.get(1).unwrap_or(&"latest");
    
    if let Some(metadata) = model_manager.get_metadata(name, tag)? {
        println!("\nModel: {}:{}", metadata.name, metadata.tag);
        println!("{:-<60}", "");
        println!("Format:              {}", metadata.format);
        println!("Family:              {}", metadata.family);
        println!("Parameter Size:      {}", metadata.parameter_size);
        println!("Quantization:        {}", metadata.quantization_level);
        println!("Size:                {:.2} MB", metadata.size as f64 / 1024.0 / 1024.0);
        println!("Path:                {}", metadata.path);
        println!("Created:             {}", metadata.created_at.format("%Y-%m-%d %H:%M:%S"));
        println!("Modified:            {}", metadata.modified_at.format("%Y-%m-%d %H:%M:%S"));
        println!("{:-<60}", "");
    } else {
        println!("Model not found: {}", model_name);
    }
    
    Ok(())
}

pub async fn list_running() -> Result<()> {
    let config = Arc::new(Config::load()?);
    let model_manager = ModelManager::new(config)?;
    
    let loaded = model_manager.list_loaded_models().await;
    
    if loaded.is_empty() {
        println!("No models currently loaded.");
    } else {
        println!("\nCurrently loaded models:");
        println!("{:-<40}", "");
        for model in loaded {
            println!("  • {}", model);
        }
        println!("{:-<40}", "");
    }
    
    Ok(())
}
