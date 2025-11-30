#!/bin/bash
# Test Ollama connection before running app

OLLAMA_URL="http://192.168.1.5:11434"

echo "🔍 Testing Ollama connection..."
echo "URL: $OLLAMA_URL"

if curl -s --max-time 5 "$OLLAMA_URL/api/tags" > /dev/null; then
    echo "✅ NR5 IS ALIVE!"
    
    echo ""
    echo "📋 Available models:"
    curl -s "$OLLAMA_URL/api/tags" | grep -o '"name":"[^"]*"' | cut -d'"' -f4
    
    exit 0
else
    echo "❌ Cannot reach Ollama at $OLLAMA_URL"
    echo ""
    echo "Troubleshooting:"
    echo "1. Check if Ollama is running on 192.168.1.5"
    echo "2. Verify network connection"
    echo "3. Check firewall settings"
    
    exit 1
fi
