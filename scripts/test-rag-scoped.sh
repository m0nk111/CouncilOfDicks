#!/bin/bash
# Test Scoped RAG functionality

echo "📝 Seeding #general memory..."
curl -s -X POST http://localhost:8080/api/chat/send \
  -H "Content-Type: application/json" \
  -d '{"content": "IMPORTANT: The secret code for GENERAL channel is BLUE_BANANA.", "channel": "general", "author": "tester", "author_type": "human"}'

echo "📝 Seeding #topic memory..."
curl -s -X POST http://localhost:8080/api/chat/send \
  -H "Content-Type: application/json" \
  -d '{"content": "IMPORTANT: The secret code for TOPIC channel is RED_APPLE.", "channel": "topic", "author": "tester", "author_type": "human"}'

echo "⏳ Waiting for embeddings (5s)..."
sleep 5

echo "❓ Asking in #general..."
curl -s -X POST http://localhost:8080/api/chat/send \
  -H "Content-Type: application/json" \
  -d '{"content": "@qwen What is the secret code?", "channel": "general", "author": "tester", "author_type": "human"}'

echo "❓ Asking in #topic..."
curl -s -X POST http://localhost:8080/api/chat/send \
  -H "Content-Type: application/json" \
  -d '{"content": "@qwen What is the secret code?", "channel": "topic", "author": "tester", "author_type": "human"}'

echo "✅ Test commands sent. Please check backend logs for AI responses."
