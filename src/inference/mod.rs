<<<<<<< HEAD
pub mod engine;
pub mod tokenizer;
pub mod sampler;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub repeat_penalty: f32,
    pub max_tokens: usize,
    pub stop_sequences: Vec<String>,
    pub stream: bool,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            temperature: 0.8,
            top_p: 0.95,
            top_k: 40,
            repeat_penalty: 1.1,
            max_tokens: 2048,
            stop_sequences: vec![],
            stream: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRequest {
    pub prompt: String,
    pub config: GenerationConfig,
    pub context: Option<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResponse {
    pub text: String,
    pub tokens_generated: usize,
    pub context: Vec<i32>,
}
=======
pub mod engine;
pub mod tokenizer;
pub mod sampler;

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub repeat_penalty: f32,
    pub max_tokens: usize,
    pub stop_sequences: Vec<String>,
    pub stream: bool,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            temperature: env::var("RUST_LLM_TEMPERATURE")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(0.8),
            top_p: env::var("RUST_LLM_TOP_P")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(0.95),
            top_k: env::var("RUST_LLM_TOP_K")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(40),
            repeat_penalty: env::var("RUST_LLM_REPEAT_PENALTY")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(1.1),
            max_tokens: env::var("RUST_LLM_MAX_TOKENS")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(2048),
            stop_sequences: vec![],
            stream: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRequest {
    pub prompt: String,
    pub config: GenerationConfig,
    pub context: Option<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResponse {
    pub text: String,
    pub tokens_generated: usize,
    pub context: Vec<i32>,
}
>>>>>>> bb9577f (20260204_220651)
