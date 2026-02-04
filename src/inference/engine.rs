<<<<<<< HEAD
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::token::data_array::LlamaTokenDataArray;
use crate::config::Config;
use crate::inference::{GenerationRequest, GenerationResponse};

pub struct InferenceEngine {
    model_path: String,
    _config: Arc<Config>,
    backend: Arc<LlamaBackend>,
    model: Arc<LlamaModel>,
    context_tokens: Arc<Mutex<Vec<i32>>>,
}

impl InferenceEngine {
    pub fn new(model_path: &str, config: Arc<Config>) -> Result<Self> {
        if !Path::new(model_path).exists() {
            anyhow::bail!("Model file not found: {}", model_path);
        }
        
        tracing::info!("Loading model from: {}", model_path);
        
        let backend = LlamaBackend::init()?;
        
        let model_params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(&backend, model_path, &model_params)?;
        
        tracing::info!("Model loaded successfully");
        
        Ok(Self {
            model_path: model_path.to_string(),
            _config: config,
            backend: Arc::new(backend),
            model: Arc::new(model),
            context_tokens: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    pub async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse> {
        let prompt = request.prompt.clone();
        let gen_config = request.config.clone();
        
        tracing::info!("Generating response for prompt (length: {})", prompt.len());
        
        let model = self.model.clone();
        let max_tokens = gen_config.max_tokens as i32;
        let temperature = gen_config.temperature;
        
        let response_text = tokio::task::spawn_blocking(move || -> Result<String> {
            let ctx_params = LlamaContextParams::default()
                .with_n_ctx(std::num::NonZeroU32::new(2048));
            let mut ctx = model.new_context(&LlamaBackend::init()?, ctx_params)?;
            
            let tokens = model.str_to_token(&prompt, llama_cpp_2::model::AddBos::Always)?;
            
            let mut batch = LlamaBatch::new(512, 1);
            for (i, token) in tokens.iter().enumerate() {
                batch.add(*token, i as i32, &[0], i == tokens.len() - 1)?;
            }
            
            ctx.decode(&mut batch)?;
            
            let mut output = String::new();
            let mut n_cur = tokens.len() as i32;
            
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32;
            
            for _ in 0..max_tokens {
                let candidates = ctx.candidates_ith(batch.n_tokens() - 1);
                let mut candidates_data = LlamaTokenDataArray::from_iter(candidates, false);
                
                candidates_data.sample_softmax(None);
                let new_token = candidates_data.sample_token(seed);
                
                if model.is_eog_token(new_token) {
                    break;
                }
                
                let piece = model.token_to_str(new_token, llama_cpp_2::model::Special::Tokenize)?;
                output.push_str(&piece);
                
                batch.clear();
                batch.add(new_token, n_cur, &[0], true)?;
                ctx.decode(&mut batch)?;
                n_cur += 1;
            }
            
            Ok(output)
        }).await??;
        
        let tokens_generated = response_text.split_whitespace().count();
        
        Ok(GenerationResponse {
            text: response_text,
            tokens_generated,
            context: vec![],
        })
    }
    
    pub async fn generate_stream(
        &self,
        request: GenerationRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<String>>> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        let model = self.model.clone();
        let prompt = request.prompt.clone();
        let gen_config = request.config.clone();
        
        tokio::task::spawn_blocking(move || -> Result<()> {
            let ctx_params = LlamaContextParams::default()
                .with_n_ctx(std::num::NonZeroU32::new(2048));
            let backend = LlamaBackend::init()?;
            let mut ctx = model.new_context(&backend, ctx_params)?;
            
            let tokens = model.str_to_token(&prompt, llama_cpp_2::model::AddBos::Always)?;
            
            let mut batch = LlamaBatch::new(512, 1);
            for (i, token) in tokens.iter().enumerate() {
                batch.add(*token, i as i32, &[0], i == tokens.len() - 1)?;
            }
            
            ctx.decode(&mut batch)?;
            
            let mut n_cur = tokens.len() as i32;
            let max_tokens = gen_config.max_tokens as i32;
            
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32;
            
            for _ in 0..max_tokens {
                let candidates = ctx.candidates_ith(batch.n_tokens() - 1);
                let mut candidates_data = LlamaTokenDataArray::from_iter(candidates, false);
                
                candidates_data.sample_softmax(None);
                let new_token = candidates_data.sample_token(seed);
                
                if model.is_eog_token(new_token) {
                    break;
                }
                
                let piece = model.token_to_str(new_token, llama_cpp_2::model::Special::Tokenize)?;
                let _ = tx.blocking_send(Ok(piece));
                
                batch.clear();
                batch.add(new_token, n_cur, &[0], true)?;
                ctx.decode(&mut batch)?;
                n_cur += 1;
            }
            
            Ok(())
        });
        
        Ok(rx)
    }
    
    pub fn get_model_path(&self) -> &str {
        &self.model_path
    }
}
=======
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::token::data_array::LlamaTokenDataArray;
use llama_cpp_2::sampling::LlamaSampler;
use crate::config::Config;
use crate::inference::{GenerationRequest, GenerationResponse};

pub struct InferenceEngine {
    model_path: String,
    _config: Arc<Config>,
    backend: Arc<LlamaBackend>,
    model: Arc<LlamaModel>,
    context_tokens: Arc<Mutex<Vec<i32>>>,
}

impl InferenceEngine {
    pub fn new(model_path: &str, config: Arc<Config>) -> Result<Self> {
        if !Path::new(model_path).exists() {
            anyhow::bail!("Model file not found: {}", model_path);
        }
        
        tracing::info!("Loading model from: {}", model_path);
        tracing::info!("🚀 Initializing with TOP PERFORMANCE settings");
        
        let backend = LlamaBackend::init()?;
        
        // GPU layers - offload everything to GPU
        let gpu_layers = Self::detect_gpu_layers();
        tracing::info!("GPU layers: {}", gpu_layers);
        
        // Performance-optimized model parameters
        let model_params = LlamaModelParams::default()
            .with_n_gpu_layers(gpu_layers);
        
        let model = LlamaModel::load_from_file(&backend, model_path, &model_params)?;
        
        tracing::info!("✓ Model loaded with GPU acceleration + mmap");
        
        Ok(Self {
            model_path: model_path.to_string(),
            _config: config,
            backend: Arc::new(backend),
            model: Arc::new(model),
            context_tokens: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    pub async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse> {
        let prompt = request.prompt.clone();
        let gen_config = request.config.clone();
        
        tracing::info!("Generating response for prompt (length: {})", prompt.len());
        
        let model = self.model.clone();
        let max_tokens = gen_config.max_tokens as i32;
        let temperature = gen_config.temperature;
        
        let response_text = tokio::task::spawn_blocking(move || -> Result<String> {
            // Performance-optimized context parameters
            let n_threads = Self::get_optimal_threads();
            let ctx_size = Self::get_env_usize("RUST_LLM_CONTEXT_SIZE", 4096);
            let batch_size = Self::get_env_usize("RUST_LLM_BATCH_SIZE", 512);
            
            let ctx_params = LlamaContextParams::default()
                .with_n_ctx(std::num::NonZeroU32::new(ctx_size as u32))
                .with_n_batch(batch_size as u32)
                .with_n_threads(n_threads as i32)
                .with_n_threads_batch(n_threads as i32);
            
            let mut ctx = model.new_context(&LlamaBackend::init()?, ctx_params)?;
            
            // Clear KV cache before starting new generation
            ctx.clear_kv_cache();
            
            let tokens = model.str_to_token(&prompt, llama_cpp_2::model::AddBos::Always)?;
            tracing::debug!("Tokenized prompt into {} tokens", tokens.len());
            
            let mut batch = LlamaBatch::new(batch_size, 1);
            for (i, token) in tokens.iter().enumerate() {
                let is_last = i == tokens.len() - 1;
                batch.add(*token, i as i32, &[0], is_last)?;
            }
            
            ctx.decode(&mut batch)?;
            
            let mut output = String::new();
            let mut n_cur = tokens.len() as i32;
            
            // High-performance sampler chain
            let temp = Self::get_env_f32("RUST_LLM_TEMPERATURE", 0.7);
            let top_p = Self::get_env_f32("RUST_LLM_TOP_P", 0.9);
            let top_k = Self::get_env_usize("RUST_LLM_TOP_K", 40) as i32;
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as u32;
            
            // Build sampler chain: top_k -> top_p -> temp -> dist
            let mut sampler = LlamaSampler::chain_simple([
                LlamaSampler::top_k(top_k),
                LlamaSampler::top_p(top_p, 1),
                LlamaSampler::temp(temp),
                LlamaSampler::dist(seed),
            ]);
            
            for _ in 0..max_tokens {
                let new_token = sampler.sample(&ctx, batch.n_tokens() - 1);
                sampler.accept(new_token);
                
                if model.is_eog_token(new_token) {
                    break;
                }
                
                let piece = model.token_to_str(new_token, llama_cpp_2::model::Special::Tokenize)?;
                output.push_str(&piece);
                
                batch.clear();
                batch.add(new_token, n_cur, &[0], true)?;
                ctx.decode(&mut batch)?;
                n_cur += 1;
            }
            
            Ok(output)
        }).await??;
        
        let tokens_generated = response_text.split_whitespace().count();
        
        Ok(GenerationResponse {
            text: response_text,
            tokens_generated,
            context: vec![],
        })
    }
    
    pub async fn generate_stream(
        &self,
        request: GenerationRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<String>>> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        let model = self.model.clone();
        let prompt = request.prompt.clone();
        let gen_config = request.config.clone();
        
        tokio::task::spawn_blocking(move || -> Result<()> {
            // Performance-optimized context parameters
            let n_threads = Self::get_optimal_threads();
            let ctx_size = Self::get_env_usize("RUST_LLM_CONTEXT_SIZE", 4096);
            let batch_size = Self::get_env_usize("RUST_LLM_BATCH_SIZE", 512);
            
            let ctx_params = LlamaContextParams::default()
                .with_n_ctx(std::num::NonZeroU32::new(ctx_size as u32))
                .with_n_batch(batch_size as u32)
                .with_n_threads(n_threads as i32)
                .with_n_threads_batch(n_threads as i32);
            
            let backend = LlamaBackend::init()?;
            let mut ctx = model.new_context(&backend, ctx_params)?;
            
            // Clear KV cache before starting new generation
            ctx.clear_kv_cache();
            
            let tokens = model.str_to_token(&prompt, llama_cpp_2::model::AddBos::Always)?;
            
            let mut batch = LlamaBatch::new(batch_size, 1);
            for (i, token) in tokens.iter().enumerate() {
                let is_last = i == tokens.len() - 1;
                batch.add(*token, i as i32, &[0], is_last)?;
            }
            
            ctx.decode(&mut batch)?;
            
            let mut n_cur = tokens.len() as i32;
            let max_tokens = gen_config.max_tokens as i32;
            
            // High-performance sampler chain
            let temp = Self::get_env_f32("RUST_LLM_TEMPERATURE", 0.7);
            let top_p = Self::get_env_f32("RUST_LLM_TOP_P", 0.9);
            let top_k = Self::get_env_usize("RUST_LLM_TOP_K", 40) as i32;
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as u32;
            
            // Build sampler chain: top_k -> top_p -> temp -> dist
            let mut sampler = LlamaSampler::chain_simple([
                LlamaSampler::top_k(top_k),
                LlamaSampler::top_p(top_p, 1),
                LlamaSampler::temp(temp),
                LlamaSampler::dist(seed),
            ]);
            
            for _ in 0..max_tokens {
                let new_token = sampler.sample(&ctx, batch.n_tokens() - 1);
                sampler.accept(new_token);
                
                if model.is_eog_token(new_token) {
                    break;
                }
                
                let piece = model.token_to_str(new_token, llama_cpp_2::model::Special::Tokenize)?;
                let _ = tx.blocking_send(Ok(piece));
                
                batch.clear();
                batch.add(new_token, n_cur, &[0], true)?;
                ctx.decode(&mut batch)?;
                n_cur += 1;
            }
            
            Ok(())
        });
        
        Ok(rx)
    }
    
    pub fn get_model_path(&self) -> &str {
        &self.model_path
    }
    
    /// Detect available GPU and return optimal number of layers to offload
    fn detect_gpu_layers() -> u32 {
        // First check environment variable override
        if let Ok(layers) = std::env::var("RUST_LLM_GPU_LAYERS") {
            if let Ok(n) = layers.parse::<i32>() {
                if n == 0 {
                    tracing::info!("GPU disabled via RUST_LLM_GPU_LAYERS=0, using CPU");
                    return 0;
                }
                tracing::info!("Using {} GPU layers from RUST_LLM_GPU_LAYERS", n);
                return n as u32;
            }
        }
        
        // Auto-detect GPU
        if Self::nvidia_gpu_available() {
            tracing::info!("NVIDIA CUDA GPU detected");
            return 999;
        }
        
        #[cfg(target_os = "macos")]
        {
            tracing::info!("Apple Metal GPU detected (macOS)");
            return 999;
        }
        
        tracing::info!("No GPU detected, using CPU");
        0
    }
    
    #[cfg(feature = "cuda")]
    fn nvidia_gpu_available() -> bool {
        // Check if nvidia-smi is available
        std::process::Command::new("nvidia-smi")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    
    #[cfg(not(feature = "cuda"))]
    fn nvidia_gpu_available() -> bool {
        false
    }
    
    /// Get optimal thread count for CPU operations
    fn get_optimal_threads() -> usize {
        // Check env override first
        if let Ok(threads) = std::env::var("RUST_LLM_THREADS") {
            if let Ok(n) = threads.parse::<usize>() {
                return n;
            }
        }
        
        // Use physical cores (not hyperthreads) for best performance
        let cpus = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);
        
        // For GPU inference, use fewer CPU threads
        // For CPU inference, use all physical cores
        if Self::nvidia_gpu_available() {
            (cpus / 2).max(4)  // Half cores for GPU mode
        } else {
            cpus  // All cores for CPU mode
        }
    }
    
    fn get_env_usize(key: &str, default: usize) -> usize {
        std::env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }
    
    fn get_env_f32(key: &str, default: f32) -> f32 {
        std::env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }
}
>>>>>>> bb9577f (20260204_220651)
