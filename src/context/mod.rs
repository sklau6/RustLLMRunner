use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ContextManager {
    contexts: Arc<RwLock<HashMap<String, Vec<i32>>>>,
    max_context_size: usize,
}

impl ContextManager {
    pub fn new(max_context_size: usize) -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            max_context_size,
        }
    }
    
    pub async fn get_context(&self, session_id: &str) -> Option<Vec<i32>> {
        let contexts = self.contexts.read().await;
        contexts.get(session_id).cloned()
    }
    
    pub async fn save_context(&self, session_id: &str, context: Vec<i32>) {
        let mut contexts = self.contexts.write().await;
        
        let trimmed_context = if context.len() > self.max_context_size {
            context[context.len() - self.max_context_size..].to_vec()
        } else {
            context
        };
        
        contexts.insert(session_id.to_string(), trimmed_context);
    }
    
    pub async fn clear_context(&self, session_id: &str) {
        let mut contexts = self.contexts.write().await;
        contexts.remove(session_id);
    }
    
    pub async fn clear_all(&self) {
        let mut contexts = self.contexts.write().await;
        contexts.clear();
    }
}
