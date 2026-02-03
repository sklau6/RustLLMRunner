#!/usr/bin/env node

/**
 * Example OpenAI client for Rust LLM Runner (Node.js)
 */

import OpenAI from 'openai';

const client = new OpenAI({
  baseURL: 'http://localhost:11434/v1',
  apiKey: 'not-needed',
});

async function chatExample() {
  console.log('=== Chat Completion Example ===\n');
  
  const response = await client.chat.completions.create({
    model: 'llama4:scout',
    messages: [
      { role: 'system', content: 'You are a helpful assistant.' },
      { role: 'user', content: 'What is Rust programming language?' }
    ],
    temperature: 0.7,
    max_tokens: 500,
  });
  
  console.log(response.choices[0].message.content);
  console.log(`\nTokens used: ${response.usage.total_tokens}`);
}

async function streamingExample() {
  console.log('\n=== Streaming Example ===\n');
  
  const stream = await client.chat.completions.create({
    model: 'llama4:scout',
    messages: [
      { role: 'user', content: 'Write a short poem about coding' }
    ],
    stream: true,
  });
  
  for await (const chunk of stream) {
    const content = chunk.choices[0]?.delta?.content;
    if (content) {
      process.stdout.write(content);
    }
  }
  
  console.log('\n');
}

async function main() {
  try {
    await chatExample();
    await streamingExample();
  } catch (error) {
    console.error('Error:', error.message);
  }
}

main();
