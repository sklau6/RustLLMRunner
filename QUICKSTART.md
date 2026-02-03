# Quick Start Guide

Get up and running with Rust LLM Runner in 5 minutes!

## Step 1: Build the Project

```bash
# Navigate to the project directory
cd rust-llm-runner

# Build in release mode (optimized)
cargo build --release
```

## Step 2: Start the Server

```bash
# Start the server on default port 11434
cargo run --release -- serve
```

You should see:
```
Server listening on http://127.0.0.1:11434
OpenAI API compatible endpoint: http://127.0.0.1:11434/v1/chat/completions
Ollama API compatible endpoint: http://127.0.0.1:11434/api/generate
```

## Step 3: Download a Model

Open a new terminal and download a model:

```bash
# Download Llama 4 Scout (recommended for testing)
cargo run --release -- pull llama4:scout

# Or download Qwen 3
cargo run --release -- pull qwen3:latest

# Or download Gemma 3 27B (larger model)
cargo run --release -- pull gemma3:27b
```

## Step 4: Test the API

### Using curl (OpenAI format):

```bash
curl http://localhost:11434/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama4:scout",
    "messages": [
      {"role": "user", "content": "Hello! How are you?"}
    ]
  }'
```

### Using curl (Ollama format):

```bash
curl http://localhost:11434/api/generate \
  -d '{
    "model": "llama4:scout",
    "prompt": "Explain what Rust is in one sentence"
  }'
```

### Using Python:

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:11434/v1",
    api_key="not-needed"
)

response = client.chat.completions.create(
    model="llama4:scout",
    messages=[{"role": "user", "content": "Hello!"}]
)

print(response.choices[0].message.content)
```

## Step 5: Interactive Chat

```bash
cargo run --release -- run llama4:scout
```

Type your messages and press Enter. Type `exit` to quit.

## Common Commands

```bash
# List all downloaded models
cargo run --release -- list

# Show model details
cargo run --release -- show llama4:scout

# List currently loaded models
cargo run --release -- ps

# Remove a model
cargo run --release -- rm llama4:scout
```

## Troubleshooting

### Port already in use
```bash
# Use a different port
cargo run --release -- serve --port 8080
```

### Model not found
```bash
# Make sure you've pulled the model first
cargo run --release -- pull llama4:scout
```

### Build errors
```bash
# Update Rust to latest stable
rustup update stable

# Clean and rebuild
cargo clean
cargo build --release
```

## Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Check out [examples/](examples/) for client code samples
- Configure GPU acceleration (see README.md)
- Explore the API endpoints

## Support

- GitHub Issues: Report bugs or request features
- Documentation: Check the README.md and wiki
- Examples: See the examples/ directory

Happy coding! 🦀
