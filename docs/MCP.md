# Model Context Protocol (MCP) Integration

Council Of Dicks implements an MCP server that allows external tools and AI assistants to query the council for consensus-based answers.

## Overview

Every Council node can act as an **MCP server**, exposing council deliberation capabilities to external clients via JSON-RPC 2.0 over TCP.

## Architecture

```
┌─────────────────┐         ┌──────────────────┐
│   MCP Client    │────────▶│  Council Node    │
│ (Claude, etc.)  │  TCP    │  (MCP Server)    │
└─────────────────┘         └──────────────────┘
                                      │
                                      ▼
                            ┌──────────────────┐
                            │  P2P Network     │
                            │  (libp2p)        │
                            └──────────────────┘
                                      │
                            ┌─────────▼─────────┐
                            │  Other AI Peers   │
                            │  (Deliberation)   │
                            └───────────────────┘
```

## MCP Tools

### 1. `council_ask`

Ask a question to the council. The council will deliberate and reach consensus through multi-round voting.

**Parameters:**
- `question` (string, required): The question to ask
- `wait_for_consensus` (boolean, optional): Wait for consensus before returning (default: false)

**Returns:**
```json
{
  "session_id": "abc123...",
  "question": "What is the meaning of life?",
  "status": "GatheringResponses",
  "message": "Council session created. Awaiting responses from AI peers."
}
```

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "council/ask",
  "params": {
    "question": "Should we implement feature X?",
    "wait_for_consensus": false
  }
}
```

### 2. `council_get_session`

Get details of a specific council session including responses, votes, and consensus status.

**Parameters:**
- `session_id` (string, required): The session ID to retrieve

**Returns:**
```json
{
  "id": "abc123...",
  "question": "What is the meaning of life?",
  "responses": [
    {
      "model_name": "qwen2.5-coder:7b",
      "response": "42",
      "peer_id": "12D3KooW...",
      "timestamp": 1701360000
    }
  ],
  "commitments": [],
  "reveals": [],
  "consensus": null,
  "status": "GatheringResponses",
  "created_at": 1701360000
}
```

### 3. `council_list_sessions`

List all council sessions with their current status.

**Returns:** Array of `CouncilSession` objects.

## Configuration

The MCP server listens on `127.0.0.1:9001` by default.

**To change the port:**
Edit `src-tauri/src/state.rs`:
```rust
mcp_server: Arc::new(McpServer::new(9001, council_manager.clone(), logger.clone())),
```

## Starting the MCP Server

### From UI
Click "Start MCP Server" in the Council UI.

### From Code
```typescript
import { mcpStart, mcpStop, mcpStatus } from './api';

// Start server
await mcpStart();

// Check status
const running = await mcpStatus();

// Stop server
await mcpStop();
```

### From Command Line
Use any MCP client or netcat:
```bash
# Test connection
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | nc localhost 9001

# Ask question
echo '{"jsonrpc":"2.0","id":2,"method":"council/ask","params":{"question":"Test?"}}' | nc localhost 9001
```

## Protocol Details

### JSON-RPC 2.0
The MCP server implements JSON-RPC 2.0 over TCP.

**Request Format:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "council/ask",
  "params": { "question": "..." }
}
```

**Response Format:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": { ... }
}
```

**Error Format:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid params"
  }
}
```

### Error Codes
- `-32700`: Parse error (invalid JSON)
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params (e.g., session not found)

## Security Considerations

⚠️ **Current Implementation:**
- Listens on `127.0.0.1` (localhost only)
- No authentication required
- Suitable for local development

🔒 **Production Recommendations:**
1. **Authentication**: Add API key or token-based auth
2. **Rate Limiting**: Prevent abuse
3. **TLS/SSL**: Encrypt communication
4. **Firewall**: Restrict access to trusted IPs
5. **Audit Logging**: Track all requests

## Use Cases

### 1. AI Assistant Integration
Claude Desktop, GPT clients, or other AI tools can query the council for consensus:
```
User: @council What's the best approach for implementing feature X?
Claude: [Uses council_ask MCP tool]
Council: [Deliberates with multiple AI models]
Claude: Based on council consensus: ...
```

### 2. Automated Decision Making
CI/CD pipelines can query the council for architectural decisions:
```bash
# In build script
decision=$(ask_council "Should we deploy this code?")
if [ "$decision" == "yes" ]; then deploy; fi
```

### 3. Research & Validation
Researchers can use the council to validate hypotheses across multiple AI models:
```python
import json
import socket

def ask_council(question):
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect(('localhost', 9001))
    
    request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "council/ask",
        "params": {"question": question}
    }
    
    sock.send((json.dumps(request) + "\n").encode())
    response = sock.recv(4096).decode()
    
    return json.loads(response)
```

## Future Enhancements

- [ ] WebSocket support for real-time updates
- [ ] Authentication & authorization
- [ ] Rate limiting
- [ ] TLS/SSL encryption
- [ ] Multi-language client libraries
- [ ] Streaming responses
- [ ] Batch requests
- [ ] Session expiration & cleanup

## Testing

```rust
// Run MCP server tests
cargo test mcp

// Test specific function
cargo test test_mcp_start_stop
```

## Debugging

Enable debug logging to see MCP requests/responses:
```typescript
await setDebug(true);
```

Logs will show:
```
🔌 MCP server listening on 127.0.0.1:9001
🔌 MCP client connected: 127.0.0.1:54321
🔌 MCP request: {"jsonrpc":"2.0","id":1,"method":"council/ask",...}
🏛️ MCP Ask: What is the meaning of life?
🔌 MCP response sent: {"jsonrpc":"2.0","id":1,"result":{...}}
```

## Contributing

When adding new MCP tools:
1. Add method to `McpRequest` enum in `src-tauri/src/mcp.rs`
2. Implement handler in `handle_request()`
3. Add tool definition to `ListTools` response
4. Update this documentation
5. Add tests

---

**Note:** This is a decentralized implementation. Each client is both an MCP server AND a council participant. This ensures no single point of failure and true peer-to-peer consensus.
