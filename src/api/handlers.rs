use axum::{
    extract::State,
    response::{IntoResponse, Response, sse::{Event, Sse}},
    Json,
    http::StatusCode,
};
use std::convert::Infallible;
use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;

use crate::api::types::*;
use crate::models::manager::ModelManager;
use crate::inference::{GenerationConfig, GenerationRequest};

pub struct AppState {
    pub model_manager: Arc<ModelManager>,
}

pub async fn chat_completions(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatCompletionRequest>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let model_parts: Vec<&str> = req.model.split(':').collect();
    let name = model_parts[0];
    let tag = model_parts.get(1).unwrap_or(&"latest");
    let safe_name = name.replace('/', "_").replace('\\', "_");
    
    let engine = state.model_manager.load_model(&safe_name, tag).await
        .map_err(|e| (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: e.to_string() })
        ))?;
    
    let prompt = req.messages.iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");
    
    let gen_config = GenerationConfig {
        temperature: req.temperature.unwrap_or(0.8),
        top_p: req.top_p.unwrap_or(0.95),
        max_tokens: req.max_tokens.unwrap_or(2048),
        stop_sequences: req.stop.unwrap_or_default(),
        stream: req.stream,
        ..Default::default()
    };
    
    if req.stream {
        let mut rx = engine.generate_stream(GenerationRequest {
            prompt,
            config: gen_config,
            context: None,
        }).await.map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() })
        ))?;
        
        let model = req.model.clone();
        let stream = async_stream::stream! {
            let id = Uuid::new_v4().to_string();
            let created = Utc::now().timestamp();
            
            while let Some(result) = rx.recv().await {
                match result {
                    Ok(text) => {
                        let chunk = ChatCompletionChunk {
                            id: id.clone(),
                            object: "chat.completion.chunk".to_string(),
                            created,
                            model: model.clone(),
                            choices: vec![ChatChoiceDelta {
                                index: 0,
                                delta: ChatMessageDelta {
                                    role: None,
                                    content: Some(text),
                                },
                                finish_reason: None,
                            }],
                        };
                        
                        let json = serde_json::to_string(&chunk).unwrap();
                        yield Ok::<_, Infallible>(Event::default().data(json));
                    }
                    Err(_) => break,
                }
            }
            
            let final_chunk = ChatCompletionChunk {
                id,
                object: "chat.completion.chunk".to_string(),
                created,
                model,
                choices: vec![ChatChoiceDelta {
                    index: 0,
                    delta: ChatMessageDelta {
                        role: None,
                        content: None,
                    },
                    finish_reason: Some("stop".to_string()),
                }],
            };
            
            let json = serde_json::to_string(&final_chunk).unwrap();
            yield Ok::<_, Infallible>(Event::default().data(json));
        };
        
        Ok(Sse::new(stream).into_response())
    } else {
        let response = engine.generate(GenerationRequest {
            prompt: prompt.clone(),
            config: gen_config,
            context: None,
        }).await.map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() })
        ))?;
        
        let completion = ChatCompletionResponse {
            id: Uuid::new_v4().to_string(),
            object: "chat.completion".to_string(),
            created: Utc::now().timestamp(),
            model: req.model,
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content: response.text,
                },
                finish_reason: "stop".to_string(),
            }],
            usage: Usage {
                prompt_tokens: prompt.split_whitespace().count(),
                completion_tokens: response.tokens_generated,
                total_tokens: prompt.split_whitespace().count() + response.tokens_generated,
            },
        };
        
        Ok(Json(completion).into_response())
    }
}

pub async fn generate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GenerateRequest>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let model_parts: Vec<&str> = req.model.split(':').collect();
    let name = model_parts[0];
    let tag = model_parts.get(1).unwrap_or(&"latest");
    let safe_name = name.replace('/', "_").replace('\\', "_");
    
    let engine = state.model_manager.load_model(&safe_name, tag).await
        .map_err(|e| (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: e.to_string() })
        ))?;
    
    let options = req.options.unwrap_or(GenerateOptions {
        temperature: Some(0.8),
        top_p: Some(0.95),
        top_k: Some(40),
        repeat_penalty: Some(1.1),
        num_predict: Some(2048),
    });
    
    let gen_config = GenerationConfig {
        temperature: options.temperature.unwrap_or(0.8),
        top_p: options.top_p.unwrap_or(0.95),
        top_k: options.top_k.unwrap_or(40),
        repeat_penalty: options.repeat_penalty.unwrap_or(1.1),
        max_tokens: options.num_predict.unwrap_or(2048),
        stream: req.stream,
        ..Default::default()
    };
    
    if req.stream {
        let mut rx = engine.generate_stream(GenerationRequest {
            prompt: req.prompt,
            config: gen_config,
            context: None,
        }).await.map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() })
        ))?;
        
        let model = req.model.clone();
        let stream = async_stream::stream! {
            while let Some(result) = rx.recv().await {
                match result {
                    Ok(text) => {
                        let response = GenerateResponse {
                            model: model.clone(),
                            created_at: Utc::now(),
                            response: text,
                            done: false,
                            context: None,
                            total_duration: None,
                            load_duration: None,
                            prompt_eval_count: None,
                            eval_count: None,
                        };
                        
                        let json = serde_json::to_string(&response).unwrap();
                        yield Ok::<_, Infallible>(Event::default().data(json));
                    }
                    Err(_) => break,
                }
            }
            
            let final_response = GenerateResponse {
                model,
                created_at: Utc::now(),
                response: String::new(),
                done: true,
                context: Some(vec![]),
                total_duration: Some(0),
                load_duration: Some(0),
                prompt_eval_count: Some(0),
                eval_count: Some(0),
            };
            
            let json = serde_json::to_string(&final_response).unwrap();
            yield Ok::<_, Infallible>(Event::default().data(json));
        };
        
        Ok(Sse::new(stream).into_response())
    } else {
        let response = engine.generate(GenerationRequest {
            prompt: req.prompt,
            config: gen_config,
            context: None,
        }).await.map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() })
        ))?;
        
        let gen_response = GenerateResponse {
            model: req.model,
            created_at: Utc::now(),
            response: response.text,
            done: true,
            context: Some(response.context),
            total_duration: Some(0),
            load_duration: Some(0),
            prompt_eval_count: Some(0),
            eval_count: Some(response.tokens_generated),
        };
        
        Ok(Json(gen_response).into_response())
    }
}

pub async fn list_models(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ListModelsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let models = state.model_manager.list_all_models()
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() })
        ))?;
    
    let model_list: Vec<ModelListItem> = models.into_iter().map(|m| {
        ModelListItem {
            name: format!("{}:{}", m.name, m.tag),
            modified_at: m.modified_at,
            size: m.size,
            digest: m.digest,
            details: ModelDetails {
                format: m.format,
                family: m.family,
                parameter_size: m.parameter_size,
                quantization_level: m.quantization_level,
            },
        }
    }).collect();
    
    Ok(Json(ListModelsResponse { models: model_list }))
}

pub async fn pull_model(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PullRequest>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let stream = async_stream::stream! {
        yield Ok::<_, Infallible>(Event::default().data(
            serde_json::to_string(&PullResponse {
                status: "pulling manifest".to_string(),
                digest: None,
                total: None,
                completed: None,
            }).unwrap()
        ));
        
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        yield Ok::<_, Infallible>(Event::default().data(
            serde_json::to_string(&PullResponse {
                status: "downloading".to_string(),
                digest: Some("sha256:abc123".to_string()),
                total: Some(1000000),
                completed: Some(500000),
            }).unwrap()
        ));
        
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        yield Ok::<_, Infallible>(Event::default().data(
            serde_json::to_string(&PullResponse {
                status: "success".to_string(),
                digest: Some("sha256:abc123".to_string()),
                total: Some(1000000),
                completed: Some(1000000),
            }).unwrap()
        ));
    };
    
    Ok(Sse::new(stream).into_response())
}

pub async fn show_model(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ShowRequest>,
) -> Result<Json<ShowResponse>, (StatusCode, Json<ErrorResponse>)> {
    let model_parts: Vec<&str> = req.name.split(':').collect();
    let name = model_parts[0];
    let tag = model_parts.get(1).unwrap_or(&"latest");
    let safe_name = name.replace('/', "_").replace('\\', "_");
    
    let metadata = state.model_manager.get_metadata(&safe_name, tag)
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() })
        ))?
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: "Model not found".to_string() })
        ))?;
    
    Ok(Json(ShowResponse {
        modelfile: format!("FROM {}\nPARAMETER temperature 0.8", metadata.path),
        parameters: "temperature 0.8\ntop_p 0.95".to_string(),
        template: "{{ .System }}\n{{ .Prompt }}".to_string(),
        details: ModelDetails {
            format: metadata.format,
            family: metadata.family,
            parameter_size: metadata.parameter_size,
            quantization_level: metadata.quantization_level,
        },
    }))
}

pub async fn delete_model(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let model_parts: Vec<&str> = req.name.split(':').collect();
    let name = model_parts[0];
    let tag = model_parts.get(1).unwrap_or(&"latest");
    let safe_name = name.replace('/', "_").replace('\\', "_");
    
    state.model_manager.delete_metadata(&safe_name, tag)
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() })
        ))?;
    
    Ok(StatusCode::OK)
}

pub async fn list_openai_models(
    State(state): State<Arc<AppState>>,
) -> Result<Json<OpenAIModelsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let models = state.model_manager.list_all_models()
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() })
        ))?;
    
    let model_list: Vec<OpenAIModel> = models.into_iter().map(|m| {
        OpenAIModel {
            id: format!("{}:{}", m.name, m.tag),
            object: "model".to_string(),
            created: m.created_at.timestamp(),
            owned_by: "local".to_string(),
        }
    }).collect();
    
    Ok(Json(OpenAIModelsResponse {
        object: "list".to_string(),
        data: model_list,
    }))
}

pub async fn ollama_chat(
    State(state): State<Arc<AppState>>,
    Json(req): Json<OllamaChatRequest>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let model_parts: Vec<&str> = req.model.split(':').collect();
    let name = model_parts[0];
    let tag = model_parts.get(1).unwrap_or(&"latest");
    
    // Use safe name for lookup
    let safe_name = name.replace('/', "_").replace('\\', "_");
    
    let engine = state.model_manager.load_model(&safe_name, tag).await
        .map_err(|e| (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: e.to_string() })
        ))?;
    
    let prompt = req.messages.iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");
    
    let options = req.options.unwrap_or(GenerateOptions {
        temperature: Some(0.8),
        top_p: Some(0.95),
        top_k: Some(40),
        repeat_penalty: Some(1.1),
        num_predict: Some(2048),
    });
    
    let gen_config = GenerationConfig {
        temperature: options.temperature.unwrap_or(0.8),
        top_p: options.top_p.unwrap_or(0.95),
        top_k: options.top_k.unwrap_or(40),
        repeat_penalty: options.repeat_penalty.unwrap_or(1.1),
        max_tokens: options.num_predict.unwrap_or(2048),
        stream: req.stream,
        ..Default::default()
    };
    
    if req.stream {
        let mut rx = engine.generate_stream(GenerationRequest {
            prompt,
            config: gen_config,
            context: None,
        }).await.map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() })
        ))?;
        
        let model = req.model.clone();
        let stream = async_stream::stream! {
            while let Some(result) = rx.recv().await {
                match result {
                    Ok(text) => {
                        let response = OllamaChatResponse {
                            model: model.clone(),
                            created_at: Utc::now(),
                            message: OllamaChatMessage {
                                role: "assistant".to_string(),
                                content: text,
                            },
                            done: false,
                            total_duration: None,
                            load_duration: None,
                            prompt_eval_count: None,
                            eval_count: None,
                        };
                        
                        let json = serde_json::to_string(&response).unwrap();
                        yield Ok::<_, Infallible>(Event::default().data(json));
                    }
                    Err(_) => break,
                }
            }
            
            let final_response = OllamaChatResponse {
                model,
                created_at: Utc::now(),
                message: OllamaChatMessage {
                    role: "assistant".to_string(),
                    content: String::new(),
                },
                done: true,
                total_duration: Some(0),
                load_duration: Some(0),
                prompt_eval_count: Some(0),
                eval_count: Some(0),
            };
            
            let json = serde_json::to_string(&final_response).unwrap();
            yield Ok::<_, Infallible>(Event::default().data(json));
        };
        
        Ok(Sse::new(stream).into_response())
    } else {
        let response = engine.generate(GenerationRequest {
            prompt: prompt.clone(),
            config: gen_config,
            context: None,
        }).await.map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() })
        ))?;
        
        let chat_response = OllamaChatResponse {
            model: req.model,
            created_at: Utc::now(),
            message: OllamaChatMessage {
                role: "assistant".to_string(),
                content: response.text,
            },
            done: true,
            total_duration: Some(0),
            load_duration: Some(0),
            prompt_eval_count: Some(prompt.split_whitespace().count()),
            eval_count: Some(response.tokens_generated),
        };
        
        Ok(Json(chat_response).into_response())
    }
}

pub async fn version() -> Json<VersionResponse> {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
