# Rust LLM Runner

A comprehensive LLM inference server built in Rust, fully compatible with Ollama's API and OpenAI's API format. Supports GGUF models with hardware acceleration via Metal (macOS) and CUDA (NVIDIA GPUs).

## Features

- ✅ **Ollama API Compatible** - Drop-in replacement for Ollama
- ✅ **OpenAI API Compatible** - Works with OpenAI client libraries
- ✅ **GGUF Model Support** - Run quantized models efficiently
- ✅ **Hardware Acceleration** - Metal (macOS) and CUDA (NVIDIA) support
- ✅ **Model Management** - Pull, list, show, and delete models
- ✅ **Streaming Responses** - Real-time token streaming
- ✅ **Context Management** - Efficient conversation context handling
- ✅ **Multiple Models** - Support for Llama, Qwen, Gemma, and more

## Supported Models

- **Llama 4** (e.g., `llama4:scout`)
- **Qwen 3** (e.g., `qwen3:latest`)
- **Gemma 3** (e.g., `gemma3:27b`)
- Any GGUF format model from HuggingFace

## Installation

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- For CUDA support: NVIDIA GPU with CUDA 12.0+
- For Metal support: macOS with Apple Silicon or AMD GPU

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/rust-llm-runner.git
cd rust-llm-runner

# Build in release mode
cargo build --release

# The binary will be at target/release/rust-llm-runner
```

## Quick Start

### 1. Start the Server

```bash
# Start on default port 11434
cargo run --release -- serve

# Or specify custom host and port
cargo run --release -- serve --host 0.0.0.0 --port 8080
```

### 2. Pull a Model

```bash
# Pull Llama 4 Scout
cargo run --release -- pull llama4:scout

# Pull Qwen 3
cargo run --release -- pull qwen3:latest

# Pull Gemma 3 27B
cargo run --release -- pull gemma3:27b
```

### 3. Run Interactive Chat

```bash
cargo run --release -- run llama4:scout
```

### 4. Generate with Prompt

```bash
cargo run --release -- run llama4:scout --prompt "Explain quantum computing"
```

## API Usage

### OpenAI Compatible API

```bash
curl http://localhost:11434/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama4:scout",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ]
  }'
```

### Streaming Response

```bash
curl http://localhost:11434/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama4:scout",
    "messages": [
      {"role": "user", "content": "Write a story"}
    ],
    "stream": true
  }'
```

### Ollama Compatible API

```bash
curl http://localhost:11434/api/generate \
  -d '{
    "model": "llama4:scout",
    "prompt": "Why is the sky blue?"
  }'
```

### List Models

```bash
curl http://localhost:11434/api/tags
```

### Show Model Info

```bash
curl http://localhost:11434/api/show \
  -d '{"name": "llama4:scout"}'
```

### Delete Model

```bash
curl -X DELETE http://localhost:11434/api/delete \
  -d '{"name": "llama4:scout"}'
```

## CLI Commands

### Server Management

```bash
# Start server
rust-llm-runner serve

# Start with custom settings
rust-llm-runner serve --host 0.0.0.0 --port 8080
```

### Model Management

```bash
# Pull a model
rust-llm-runner pull llama4:scout

# List all models
rust-llm-runner list

# Show model details
rust-llm-runner show llama4:scout

# Remove a model
rust-llm-runner rm llama4:scout

# List running models
rust-llm-runner ps
```

### Interactive Mode

```bash
# Run model interactively
rust-llm-runner run llama4:scout

# Run with a single prompt
rust-llm-runner run llama4:scout --prompt "Your question here"
```

## Configuration

Models and data are stored in `~/.rust-llm-runner/`:

```
~/.rust-llm-runner/
├── models/          # Downloaded GGUF models
├── cache/           # Temporary cache
└── db/              # Model metadata database
```

## Hardware Acceleration

### CUDA (NVIDIA)

The runner automatically detects NVIDIA GPUs and uses CUDA acceleration when available.

Requirements:
- NVIDIA GPU with Compute Capability 6.0+
- CUDA Toolkit 12.0+
- nvidia-smi available in PATH

### Metal (macOS)

Automatically enabled on macOS with Apple Silicon or AMD GPUs.

Requirements:
- macOS 12.0+
- Apple Silicon (M1/M2/M3) or AMD GPU

### CPU Fallback

If no GPU is detected, the runner falls back to CPU inference.

## Python Client Example

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:11434/v1",
    api_key="not-needed"
)

response = client.chat.completions.create(
    model="llama4:scout",
    messages=[
        {"role": "user", "content": "Hello!"}
    ]
)

print(response.choices[0].message.content)
```

## JavaScript/TypeScript Client Example

```typescript
import OpenAI from 'openai';

const client = new OpenAI({
  baseURL: 'http://localhost:11434/v1',
  apiKey: 'not-needed',
});

const response = await client.chat.completions.create({
  model: 'llama4:scout',
  messages: [
    { role: 'user', content: 'Hello!' }
  ],
});

console.log(response.choices[0].message.content);
```

## Architecture

```
rust-llm-runner/
├── src/
│   ├── main.rs              # Entry point and CLI
│   ├── api/                 # HTTP API server
│   │   ├── server.rs        # Server initialization
│   │   ├── routes.rs        # Route definitions
│   │   ├── handlers.rs      # Request handlers
│   │   └── types.rs         # API types
│   ├── models/              # Model management
│   │   ├── registry.rs      # Model registry
│   │   ├── metadata.rs      # Metadata storage
│   │   └── manager.rs       # Model lifecycle
│   ├── inference/           # Inference engine
│   │   ├── engine.rs        # Core inference
│   │   ├── tokenizer.rs     # Tokenization
│   │   └── sampler.rs       # Sampling strategies
│   ├── download/            # Model downloading
│   ├── hardware/            # GPU acceleration
│   ├── context/             # Context management
│   ├── config/              # Configuration
│   └── cli/                 # CLI commands
└── Cargo.toml
```

## Performance Tips

1. **GPU Layers**: By default, all layers are offloaded to GPU (-1). Adjust in config if needed.
2. **Context Size**: Default is 2048 tokens. Increase for longer conversations.
3. **Quantization**: Q4_K_M offers best balance of speed and quality.
4. **Batch Size**: Adjust based on available VRAM/RAM.

## Troubleshooting

### CUDA Not Detected

```bash
# Check NVIDIA driver
nvidia-smi

# Ensure CUDA is in PATH
export PATH=/usr/local/cuda/bin:$PATH
export LD_LIBRARY_PATH=/usr/local/cuda/lib64:$LD_LIBRARY_PATH
```

### Model Download Fails

- Check internet connection
- Verify HuggingFace is accessible
- Try downloading manually and placing in `~/.rust-llm-runner/models/`

### Out of Memory

- Use smaller models or lower quantization
- Reduce context size
- Reduce number of GPU layers

## Roadmap

- [ ] Full llama.cpp integration for production inference
- [ ] Support for more model formats (SafeTensors, PyTorch)
- [ ] Model fine-tuning capabilities
- [ ] Distributed inference across multiple GPUs
- [ ] Web UI for model management
- [ ] Docker container support
- [ ] Embeddings API endpoint
- [ ] Function calling support

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Acknowledgments

- [Ollama](https://ollama.ai) for API design inspiration
- [llama.cpp](https://github.com/ggerganov/llama.cpp) for GGUF format
- [Candle](https://github.com/huggingface/candle) for ML framework
- HuggingFace for model hosting

## Support

For issues and questions:
- GitHub Issues: [Create an issue](https://github.com/yourusername/rust-llm-runner/issues)
- Documentation: [Wiki](https://github.com/yourusername/rust-llm-runner/wiki)
"# RustLLMRunner" 
