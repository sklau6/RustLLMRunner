#!/usr/bin/env python3
"""
Example OpenAI client for Rust LLM Runner
"""

from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:11434/v1",
    api_key="not-needed"
)

def chat_example():
    print("=== Chat Completion Example ===\n")
    
    response = client.chat.completions.create(
        model="llama4:scout",
        messages=[
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": "What is Rust programming language?"}
        ],
        temperature=0.7,
        max_tokens=500
    )
    
    print(response.choices[0].message.content)
    print(f"\nTokens used: {response.usage.total_tokens}")

def streaming_example():
    print("\n=== Streaming Example ===\n")
    
    stream = client.chat.completions.create(
        model="llama4:scout",
        messages=[
            {"role": "user", "content": "Write a short poem about coding"}
        ],
        stream=True
    )
    
    for chunk in stream:
        if chunk.choices[0].delta.content:
            print(chunk.choices[0].delta.content, end='', flush=True)
    
    print("\n")

if __name__ == "__main__":
    chat_example()
    streaming_example()
