use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{
    accept_async, connect_async,
    tungstenite::{Error as WsError, Message},
    MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, error, info, warn};

use super::{
    client::Transport, server::ProtocolServer, ProtocolMessage, ProtocolServerTrait,
};

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// WebSocket client implementation
pub struct WebSocketClient {
    ws: Arc<RwLock<Option<WsStream>>>,
    rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<ProtocolMessage>>>>,
    tx: mpsc::UnboundedSender<ProtocolMessage>,
}

impl WebSocketClient {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            ws: Arc::new(RwLock::new(None)),
            rx: Arc::new(RwLock::new(Some(rx))),
            tx,
        }
    }
    
    async fn start_receive_loop(&self) {
        let ws_clone = Arc::clone(&self.ws);
        let tx = self.tx.clone();
        
        tokio::spawn(async move {
            loop {
                let mut ws_guard = ws_clone.write().await;
                if let Some(ws) = ws_guard.as_mut() {
                    match ws.next().await {
                        Some(Ok(Message::Text(text))) => {
                            match serde_json::from_str::<ProtocolMessage>(&text) {
                                Ok(msg) => {
                                    if let Err(e) = tx.send(msg) {
                                        error!("Failed to send message to channel: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse message: {}", e);
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("WebSocket closed");
                            *ws_guard = None;
                            break;
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            *ws_guard = None;
                            break;
                        }
                        _ => {}
                    }
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        });
    }
}

impl Transport for WebSocketClient {
    async fn send(&self, message: ProtocolMessage) -> Result<()> {
        let mut ws_guard = self.ws.write().await;
        if let Some(ws) = ws_guard.as_mut() {
            let json = serde_json::to_string(&message)?;
            ws.send(Message::Text(json)).await
                .context("Failed to send WebSocket message")?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("WebSocket not connected"))
        }
    }
    
    async fn receive(&self) -> Result<ProtocolMessage> {
        let mut rx_guard = self.rx.write().await;
        if let Some(rx) = rx_guard.as_mut() {
            rx.recv().await
                .context("Failed to receive message from channel")
        } else {
            Err(anyhow::anyhow!("Receiver not available"))
        }
    }
    
    async fn connect(&self, url: &str) -> Result<()> {
        let (ws_stream, _) = connect_async(url).await
            .context("Failed to connect to WebSocket")?;
        
        *self.ws.write().await = Some(ws_stream);
        self.start_receive_loop().await;
        
        Ok(())
    }
    
    async fn disconnect(&self) -> Result<()> {
        let mut ws_guard = self.ws.write().await;
        if let Some(mut ws) = ws_guard.take() {
            ws.close(None).await?;
        }
        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        self.ws.read().await.is_some()
    }
}

/// WebSocket server implementation
pub struct WebSocketServer {
    server: Arc<dyn ProtocolServerTrait>,
    listener: Option<TcpListener>,
}

impl WebSocketServer {
    pub fn new(server: Arc<dyn ProtocolServerTrait>) -> Self {
        Self {
            server,
            listener: None,
        }
    }
    
    pub async fn bind(&mut self, addr: &str) -> Result<()> {
        let listener = TcpListener::bind(addr).await
            .context("Failed to bind to address")?;
        self.listener = Some(listener);
        Ok(())
    }
    
    pub async fn start(&self) -> Result<()> {
        let listener = self.listener.as_ref()
            .context("Server not bound to an address")?;
        
        info!("WebSocket server listening");
        
        loop {
            let (stream, addr) = listener.accept().await?;
            info!("New connection from: {}", addr);
            
            let ws_stream = accept_async(stream).await?;
            let server = Arc::clone(&self.server);
            
            tokio::spawn(async move {
                if let Err(e) = handle_connection(ws_stream, server).await {
                    error!("Connection error: {}", e);
                }
            });
        }
    }
}

async fn handle_connection(
    mut ws_stream: WebSocketStream<tokio::net::TcpStream>,
    server: Arc<dyn ProtocolServerTrait>,
) -> Result<()> {
    while let Some(msg) = ws_stream.next().await {
        match msg? {
            Message::Text(text) => {
                let request: ProtocolMessage = serde_json::from_str(&text)?;
                
                // Process the message based on its type
                match request.msg_type {
                    super::MessageType::Request => {
                        let response = server
                            .handle_call(&request.method, request.payload)
                            .await;
                        
                        let response_msg = match response {
                            Ok(payload) => ProtocolMessage {
                                id: request.id,
                                method: request.method,
                                msg_type: super::MessageType::Response,
                                payload,
                            },
                            Err(e) => ProtocolMessage {
                                id: request.id,
                                method: request.method,
                                msg_type: super::MessageType::Error,
                                payload: serde_json::json!({
                                    "message": e.to_string(),
                                }),
                            },
                        };
                        
                        let response_text = serde_json::to_string(&response_msg)?;
                        ws_stream.send(Message::Text(response_text)).await?;
                    }
                    super::MessageType::Stream => {
                        match server.handle_stream(&request.method, request.payload).await {
                            Ok(mut stream) => {
                                while let Some(item) = stream.next().await {
                                    let msg = match item {
                                        Ok(payload) => ProtocolMessage {
                                            id: request.id,
                                            method: request.method.clone(),
                                            msg_type: super::MessageType::StreamData,
                                            payload,
                                        },
                                        Err(e) => ProtocolMessage {
                                            id: request.id,
                                            method: request.method.clone(),
                                            msg_type: super::MessageType::Error,
                                            payload: serde_json::json!({
                                                "message": e.to_string(),
                                            }),
                                        },
                                    };
                                    
                                    let msg_text = serde_json::to_string(&msg)?;
                                    ws_stream.send(Message::Text(msg_text)).await?;
                                }
                                
                                // Send stream end message
                                let end_msg = ProtocolMessage {
                                    id: request.id,
                                    method: request.method,
                                    msg_type: super::MessageType::StreamEnd,
                                    payload: serde_json::json!({}),
                                };
                                let end_text = serde_json::to_string(&end_msg)?;
                                ws_stream.send(Message::Text(end_text)).await?;
                            }
                            Err(e) => {
                                let error_msg = ProtocolMessage {
                                    id: request.id,
                                    method: request.method,
                                    msg_type: super::MessageType::Error,
                                    payload: serde_json::json!({
                                        "message": e.to_string(),
                                    }),
                                };
                                let error_text = serde_json::to_string(&error_msg)?;
                                ws_stream.send(Message::Text(error_text)).await?;
                            }
                        }
                    }
                    _ => {
                        warn!("Unexpected message type: {:?}", request.msg_type);
                    }
                }
            }
            Message::Close(_) => {
                info!("Client disconnected");
                break;
            }
            _ => {}
        }
    }
    
    Ok(())
}