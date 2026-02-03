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
            
            for _ in 0..max_tokens {
                let candidates = ctx.candidates_ith(batch.n_tokens() - 1);
                let mut candidates_data = LlamaTokenDataArray::from_iter(candidates, false);
                
                candidates_data.sample_temp(temperature);
                let new_token = candidates_data.sample_token(&mut ctx);
                
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
            let temperature = gen_config.temperature;
            
            for _ in 0..max_tokens {
                let candidates = ctx.candidates_ith(batch.n_tokens() - 1);
                let mut candidates_data = LlamaTokenDataArray::from_iter(candidates, false);
                
                candidates_data.sample_temp(temperature);
                let new_token = candidates_data.sample_token(&mut ctx);
                
                if model.is_eog_token(new_token) {
                    break;
                }
                
                let piece = model.token_to_str(new_token, llama_cpp_2::model::Special::Tokenize)?;
                
                let tx_clone = tx.clone();
                let piece_clone = piece.clone();
                let _ = std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let _ = tx_clone.send(Ok(piece_clone)).await;
                    });
                }).join();
                
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
