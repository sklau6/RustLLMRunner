#!/bin/bash

# Example Ollama API client for Rust LLM Runner

BASE_URL="http://localhost:11434"

echo "=== Generate Example ==="
curl $BASE_URL/api/generate -d '{
  "model": "llama4:scout",
  "prompt": "Why is the sky blue?",
  "stream": false
}'

echo -e "\n\n=== Streaming Generate Example ==="
curl $BASE_URL/api/generate -d '{
  "model": "llama4:scout",
  "prompt": "Tell me a joke",
  "stream": true
}'

echo -e "\n\n=== List Models ==="
curl $BASE_URL/api/tags

echo -e "\n\n=== Show Model ==="
curl $BASE_URL/api/show -d '{
  "name": "llama4:scout"
}'
