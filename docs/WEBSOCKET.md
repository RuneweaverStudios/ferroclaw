# WebSocket Server Implementation

## Overview

The WebSocket server provides real-time event broadcasting for the Ferroclaw agent system. It enables clients to receive live updates about agent state changes and tool execution.

## Architecture

### Components

1. **WebSocket Server** (`src/websocket/mod.rs`)
   - Listens on `ws://localhost:8420`
   - Handles client connections
   - Broadcasts events to all connected clients

2. **Event Types** (`WsEvent` enum)
   - `AGENT_STATE_UPDATE`: Agent state changes (idle, thinking, executing, error)
   - `TOOL_CALL_START`: Tool execution begins
   - `TOOL_CALL_UPDATE`: Tool state changes (pending, running, completed, failed)
   - `TOOL_OUTPUT_CHUNK`: Streaming output from tool execution

3. **Integration Points**
   - `src/gateway.rs`: Gateway management with WebSocket broadcaster
   - `src/agent/loop.rs`: Agent loop emits WebSocket events during execution

### Event Flow

```
Agent Loop → Event Created → WsBroadcaster → All Connected Clients
```

## Usage

### Starting the Server

```rust
use ferroclaw::gateway;
use ferroclaw::config::Config;

let config = Config::default();
let gateway_handle = gateway::start_gateway(&config).await?;

// The server is now running on ws://localhost:8420
```

### Broadcasting Events

```rust
use ferroclaw::websocket::{WsBroadcaster, WsEvent, AgentState};

// Get the broadcaster from the gateway
let broadcaster = gateway_handle.ws_broadcaster;

// Broadcast an agent state update
let event = WsEvent::agent_state(
    "agent-123".to_string(),
    AgentState::Thinking
);
broadcaster.broadcast(event)?;
```

### Client Connection (JavaScript)

```javascript
const ws = new WebSocket('ws://localhost:8420');

ws.onopen = () => {
    console.log('Connected to WebSocket server');
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);

    switch (data.type) {
        case 'agent_state_update':
            console.log(`Agent ${data.data.agent_id} is now ${data.data.state}`);
            break;
        case 'tool_call_start':
            console.log(`Tool ${data.data.tool_name} started`);
            break;
        case 'tool_call_update':
            console.log(`Tool ${data.data.call_id} is now ${data.data.state}`);
            break;
        case 'tool_output_chunk':
            console.log(`Output: ${data.data.chunk}`);
            break;
    }
};

ws.onerror = (error) => {
    console.error('WebSocket error:', error);
};

ws.onclose = () => {
    console.log('Disconnected from WebSocket server');
};
```

### Client Connection (Python)

```python
import asyncio
import websockets
import json

async def websocket_client():
    uri = "ws://localhost:8420"
    async with websockets.connect(uri) as websocket:
        print("Connected to WebSocket server")

        while True:
            message = await websocket.recv()
            data = json.loads(message)

            if data['type'] == 'agent_state_update':
                print(f"Agent {data['data']['agent_id']} is {data['data']['state']}")
            elif data['type'] == 'tool_call_start':
                print(f"Tool {data['data']['tool_name']} started")

asyncio.run(websocket_client())
```

## Event Format

### Agent State Update

```json
{
    "type": "agent_state_update",
    "data": {
        "agent_id": "agent-123",
        "state": "thinking",
        "timestamp": 1234567890
    }
}
```

States: `idle`, `thinking`, `executing`, `error`

### Tool Call Start

```json
{
    "type": "tool_call_start",
    "data": {
        "call_id": "call-456",
        "tool_name": "read_file",
        "arguments": {
            "path": "/tmp/file.txt"
        },
        "timestamp": 1234567890
    }
}
```

### Tool Call Update

```json
{
    "type": "tool_call_update",
    "data": {
        "call_id": "call-456",
        "state": "running",
        "timestamp": 1234567890
    }
}
```

States: `pending`, `running`, `completed`, `failed`

### Tool Output Chunk

```json
{
    "type": "tool_output_chunk",
    "data": {
        "call_id": "call-456",
        "chunk": "Line 1 of output\n",
        "is_final": false,
        "timestamp": 1234567890
    }
}
```

## Running the Demo

Build and run the WebSocket demo:

```bash
cargo run --example websocket_demo
```

Then connect with a WebSocket client:

```bash
# Using wscat (npm install -g wscat)
wscat -c ws://127.0.0.1:8420
```

## Testing

Run the WebSocket integration tests:

```bash
cargo test --test integration_websocket
```

## Configuration

The WebSocket server uses the same configuration as the HTTP gateway:

```toml
[gateway]
bind = "127.0.0.1"  # WebSocket server binds here
port = 8420         # WebSocket port
```

## Security Considerations

- The WebSocket server binds to `127.0.0.1` by default (localhost only)
- No authentication is required for WebSocket connections
- For production use, consider:
  - Adding authentication tokens
  - Using TLS/WSS for encrypted connections
  - Implementing rate limiting
  - Validating and sanitizing all client messages

## Implementation Details

### Broadcast Channel

The server uses `tokio::sync::broadcast` for efficient fan-out to multiple clients:
- Channel capacity: 1000 messages
- No receivers = messages are dropped (not an error)
- Each subscriber gets their own receiver

### Connection Handling

- Each client connection runs in its own tokio task
- Ping/pong is handled automatically by tungstenite
- Graceful disconnect on client close
- Automatic cleanup on connection errors

### Performance

- Non-blocking async I/O using tokio
- Efficient JSON serialization with serde
- Zero-copy message forwarding where possible
- Minimal per-connection overhead

## Future Enhancements

Potential improvements:
- Authentication/authorization
- Message filtering by subscription
- Connection health monitoring
- Reconnection strategies
- Message persistence/replay
- Metrics and monitoring
