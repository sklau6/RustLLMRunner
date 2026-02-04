use anyhow::Result;
use std::sync::Arc;
use std::io::{Write, stdout};
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
    
    // Use async resolve to support dynamic GGUF discovery
    let url = registry.resolve_model_url(model_name).await?;
    
    // Generate a clean filename from model name
    let safe_name = name.replace('/', "_").replace('\\', "_");
    let model_path = config.get_model_path(&format!("{}_{}.gguf", safe_name, tag));
    
    println!("Downloading from: {}", url);
    downloader.download_file(&url, &model_path).await?;
    
    let file_size = tokio::fs::metadata(&model_path).await?.len();
    
    let metadata = ModelMetadata {
        name: safe_name.clone(),
        tag: tag.to_string(),
        size: file_size,
        digest: format!("sha256:{}", uuid::Uuid::new_v4()),
        format: "gguf".to_string(),
        family: safe_name.clone(),
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
    let stream_mode = config.stream_mode;
    
    let model_parts: Vec<&str> = model_name.split(':').collect();
    let name = model_parts[0];
    let tag = model_parts.get(1).unwrap_or(&"latest");
    
    // Use safe name for lookup (consistent with pull)
    let safe_name = name.replace('/', "_").replace('\\', "_");
    
    println!("Loading model: {}...", model_name);
    println!("Stream mode: {}", if stream_mode { "enabled" } else { "disabled" });
    let engine = model_manager.load_model(&safe_name, tag).await?;
    
    if let Some(p) = prompt {
        let start = std::time::Instant::now();
        
        if stream_mode {
            // Stream mode - print tokens as they arrive
            print!("\n");
            let mut rx = engine.generate_stream(GenerationRequest {
                prompt: p,
                config: GenerationConfig::default(),
                context: None,
            }).await?;
            
            let mut token_count = 0;
            while let Some(result) = rx.recv().await {
                match result {
                    Ok(token) => {
                        print!("{}", token);
                        stdout().flush()?;
                        token_count += 1;
                    }
                    Err(e) => eprintln!("\nError: {}", e),
                }
            }
            let elapsed = start.elapsed();
            println!("\n\n[⏱ {:.2}s | {} tokens | {:.1} t/s]", 
                elapsed.as_secs_f64(),
                token_count,
                token_count as f64 / elapsed.as_secs_f64()
            );
        } else {
            // Non-stream mode - wait for full response
            let response = engine.generate(GenerationRequest {
                prompt: p,
                config: GenerationConfig::default(),
                context: None,
            }).await?;
            let elapsed = start.elapsed();
            
            println!("\n{}", response.text);
            println!("\n[⏱ {:.2}s | {} tokens | {:.1} t/s]", 
                elapsed.as_secs_f64(),
                response.tokens_generated,
                response.tokens_generated as f64 / elapsed.as_secs_f64()
            );
        }
    } else {
        println!("\nInteractive mode. Type 'exit' to quit.\n");
        
        loop {
            let input: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt(">>>")
                .interact_text()?;
            
            if input.trim().eq_ignore_ascii_case("exit") {
                break;
            }
            
            let start = std::time::Instant::now();
            
            if stream_mode {
                // Stream mode
                print!("\n");
                let mut rx = engine.generate_stream(GenerationRequest {
                    prompt: input,
                    config: GenerationConfig::default(),
                    context: None,
                }).await?;
                
                let mut token_count = 0;
                while let Some(result) = rx.recv().await {
                    match result {
                        Ok(token) => {
                            print!("{}", token);
                            stdout().flush()?;
                            token_count += 1;
                        }
                        Err(e) => eprintln!("\nError: {}", e),
                    }
                }
                let elapsed = start.elapsed();
                println!("\n\n[⏱ {:.2}s | {} tokens | {:.1} t/s]\n", 
                    elapsed.as_secs_f64(),
                    token_count,
                    token_count as f64 / elapsed.as_secs_f64()
                );
            } else {
                // Non-stream mode
                let response = engine.generate(GenerationRequest {
                    prompt: input,
                    config: GenerationConfig::default(),
                    context: None,
                }).await?;
                let elapsed = start.elapsed();
                
                println!("\n{}", response.text);
                println!("\n[⏱ {:.2}s | {} tokens | {:.1} t/s]\n", 
                    elapsed.as_secs_f64(),
                    response.tokens_generated,
                    response.tokens_generated as f64 / elapsed.as_secs_f64()
                );
            }
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
    let safe_name = name.replace('/', "_").replace('\\', "_");
    
    if let Some(metadata) = model_manager.get_metadata(&safe_name, tag)? {
        let model_path = std::path::Path::new(&metadata.path);
        if model_path.exists() {
            tokio::fs::remove_file(model_path).await?;
        }
        
        model_manager.delete_metadata(&safe_name, tag)?;
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
    let safe_name = name.replace('/', "_").replace('\\', "_");
    
    if let Some(metadata) = model_manager.get_metadata(&safe_name, tag)? {
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
