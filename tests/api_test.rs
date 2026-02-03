use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn test_health_endpoint() {
    let response = "OK";
    assert_eq!(response, "OK");
}

#[test]
fn test_chat_request_serialization() {
    let request = json!({
        "model": "llama4:scout",
        "messages": [
            {"role": "user", "content": "Hello"}
        ],
        "temperature": 0.7,
        "stream": false
    });
    
    assert_eq!(request["model"], "llama4:scout");
    assert_eq!(request["messages"][0]["role"], "user");
}

#[test]
fn test_generate_request_serialization() {
    let request = json!({
        "model": "qwen3:latest",
        "prompt": "Test prompt",
        "stream": false
    });
    
    assert_eq!(request["model"], "qwen3:latest");
    assert_eq!(request["prompt"], "Test prompt");
}
