// Model Context Protocol (MCP) Server Implementation
// Allows external tools to query the Council via MCP

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::council::CouncilSessionManager;
use crate::logger::Logger;

/// MCP Request from client
#[derive(Debug, Deserialize)]
#[serde(tag = "method")]
enum McpRequest {
    #[serde(rename = "council/ask")]
    Ask { id: u64, params: AskParams },
    
    #[serde(rename = "council/get_session")]
    GetSession { id: u64, params: GetSessionParams },
    
    #[serde(rename = "council/list_sessions")]
    ListSessions { id: u64 },
    
    #[serde(rename = "tools/list")]
    ListTools { id: u64 },
}

#[derive(Debug, Deserialize)]
struct AskParams {
    question: String,
    #[serde(default)]
    wait_for_consensus: bool,
}

#[derive(Debug, Deserialize)]
struct GetSessionParams {
    session_id: String,
}

/// MCP Response to client
#[derive(Debug, Serialize)]
struct McpResponse {
    jsonrpc: String,
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<McpError>,
}

#[derive(Debug, Serialize)]
struct McpError {
    code: i32,
    message: String,
}

/// MCP Tool definition
#[derive(Debug, Serialize)]
struct McpTool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: serde_json::Value,
}

/// MCP Server
pub struct McpServer {
    port: u16,
    council_manager: Arc<CouncilSessionManager>,
    logger: Arc<Logger>,
    listener: Arc<Mutex<Option<TcpListener>>>,
}

impl McpServer {
    /// Create new MCP server
    pub fn new(port: u16, council_manager: Arc<CouncilSessionManager>, logger: Arc<Logger>) -> Self {
        Self {
            port,
            council_manager,
            logger,
            listener: Arc::new(Mutex::new(None)),
        }
    }

    /// Start MCP server
    pub async fn start(&self) -> Result<String, String> {
        let mut listener_guard = self.listener.lock().await;
        
        if listener_guard.is_some() {
            return Err("MCP server already running".to_string());
        }

        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("Failed to bind MCP server: {}", e))?;

        self.logger.success("mcp_server", &format!("MCP server listening on {}", addr));
        
        *listener_guard = Some(listener);
        Ok(format!("MCP server started on {}", addr))
    }

    /// Stop MCP server
    pub async fn stop(&self) -> Result<String, String> {
        let mut listener_guard = self.listener.lock().await;
        
        if listener_guard.is_none() {
            return Err("MCP server not running".to_string());
        }

        *listener_guard = None;
        self.logger.info("mcp_server", "MCP server stopped");
        
        Ok("MCP server stopped".to_string())
    }

    /// Check if server is running
    pub async fn is_running(&self) -> bool {
        self.listener.lock().await.is_some()
    }

    /// Accept and handle connections (call this in a loop)
    pub async fn accept_connection(&self) -> Result<(), String> {
        let listener_guard = self.listener.lock().await;
        
        if let Some(listener) = listener_guard.as_ref() {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    self.logger.debug("mcp_server", &format!("MCP client connected: {}", addr));
                    drop(listener_guard); // Release lock before handling
                    
                    let council_manager = self.council_manager.clone();
                    let logger = self.logger.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream, council_manager, logger.clone()).await {
                            logger.error("mcp_client", &format!("MCP client error: {}", e));
                        }
                    });
                    
                    Ok(())
                }
                Err(e) => Err(format!("Failed to accept connection: {}", e)),
            }
        } else {
            Err("MCP server not running".to_string())
        }
    }

    /// Handle individual client connection
    async fn handle_client(
        stream: TcpStream,
        council_manager: Arc<CouncilSessionManager>,
        logger: Arc<Logger>,
    ) -> Result<(), String> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // Connection closed
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    logger.debug("mcp_server", &format!("MCP request: {}", trimmed));

                    // Parse request
                    let response = match serde_json::from_str::<McpRequest>(trimmed) {
                        Ok(req) => Self::handle_request(req, council_manager.clone(), logger.clone()).await,
                        Err(e) => McpResponse {
                            jsonrpc: "2.0".to_string(),
                            id: 0,
                            result: None,
                            error: Some(McpError {
                                code: -32700,
                                message: format!("Parse error: {}", e),
                            }),
                        },
                    };

                    // Send response
                    let response_json = serde_json::to_string(&response)
                        .map_err(|e| format!("Failed to serialize response: {}", e))?;
                    
                    writer.write_all(response_json.as_bytes()).await
                        .map_err(|e| format!("Failed to write response: {}", e))?;
                    writer.write_all(b"\n").await
                        .map_err(|e| format!("Failed to write newline: {}", e))?;
                    
                    logger.debug("mcp_server", &format!("MCP response sent: {}", response_json));
                }
                Err(e) => {
                    logger.error("mcp_client", &format!("Failed to read from client: {}", e));
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle MCP request
    async fn handle_request(
        request: McpRequest,
        council_manager: Arc<CouncilSessionManager>,
        logger: Arc<Logger>,
    ) -> McpResponse {
        match request {
            McpRequest::Ask { id, params } => {
                logger.info("mcp_handler", &format!("MCP Ask: {}", params.question));
                
                let session_id = council_manager.create_session(params.question.clone()).await;
                
                // TODO: If wait_for_consensus, wait for consensus to be reached
                // For now, just return session ID
                
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(serde_json::json!({
                        "session_id": session_id,
                        "question": params.question,
                        "status": "GatheringResponses",
                        "message": "Council session created. Awaiting responses from AI peers."
                    })),
                    error: None,
                }
            }
            
            McpRequest::GetSession { id, params } => {
                logger.debug("mcp_handler", &format!("MCP GetSession: {}", params.session_id));
                
                match council_manager.get_session(&params.session_id).await {
                    Some(session) => McpResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: Some(serde_json::to_value(&session).unwrap()),
                        error: None,
                    },
                    None => McpResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(McpError {
                            code: -32602,
                            message: "Session not found".to_string(),
                        }),
                    },
                }
            }
            
            McpRequest::ListSessions { id } => {
                logger.debug("mcp_handler", "MCP ListSessions");
                
                let sessions = council_manager.list_sessions().await;
                
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(serde_json::to_value(&sessions).unwrap()),
                    error: None,
                }
            }
            
            McpRequest::ListTools { id } => {
                logger.debug("mcp_handler", "MCP ListTools");
                
                let tools = vec![
                    McpTool {
                        name: "council_ask".to_string(),
                        description: "Ask a question to the Council of AI models. The council will deliberate and reach consensus through multi-round voting.".to_string(),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "question": {
                                    "type": "string",
                                    "description": "The question to ask the council"
                                },
                                "wait_for_consensus": {
                                    "type": "boolean",
                                    "description": "Wait for consensus to be reached before returning",
                                    "default": false
                                }
                            },
                            "required": ["question"]
                        }),
                    },
                    McpTool {
                        name: "council_get_session".to_string(),
                        description: "Get details of a specific council session including responses, votes, and consensus status.".to_string(),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "session_id": {
                                    "type": "string",
                                    "description": "The ID of the session to retrieve"
                                }
                            },
                            "required": ["session_id"]
                        }),
                    },
                    McpTool {
                        name: "council_list_sessions".to_string(),
                        description: "List all council sessions with their current status.".to_string(),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {}
                        }),
                    },
                ];
                
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(serde_json::json!({ "tools": tools })),
                    error: None,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let council_manager = Arc::new(CouncilSessionManager::new());
        let logger = Arc::new(Logger::new(false));
        let mcp = McpServer::new(9001, council_manager, logger);
        
        assert!(!mcp.is_running().await);
    }

    #[tokio::test]
    async fn test_mcp_start_stop() {
        let council_manager = Arc::new(CouncilSessionManager::new());
        let logger = Arc::new(Logger::new(false));
        let mcp = McpServer::new(9001, council_manager, logger);
        
        // Start server
        let result = mcp.start().await;
        assert!(result.is_ok());
        assert!(mcp.is_running().await);
        
        // Stop server
        let result = mcp.stop().await;
        assert!(result.is_ok());
        assert!(!mcp.is_running().await);
    }

    #[tokio::test]
    async fn test_mcp_double_start() {
        let council_manager = Arc::new(CouncilSessionManager::new());
        let logger = Arc::new(Logger::new(false));
        let mcp = McpServer::new(9002, council_manager, logger);
        
        mcp.start().await.unwrap();
        let result = mcp.start().await;
        assert!(result.is_err());
        
        mcp.stop().await.unwrap();
    }
}
