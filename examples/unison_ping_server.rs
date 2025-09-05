use anyhow::Result;
use unison_protocol::{UnisonProtocol, UnisonServer, UnisonServerExt, ProtocolServer};
use unison_protocol::network::NetworkError;
use serde_json::json;
use std::time::{Duration, Instant};
use tracing::{info, Level};
use tracing_subscriber;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🎵 Unison Protocol Ping Server Starting...");
    
    // Create Unison protocol instance
    let mut protocol = UnisonProtocol::new();
    
    // Load the ping-pong protocol schema
    protocol.load_schema(include_str!("../schemas/ping_pong.kdl"))?;
    
    // Create server
    let mut server = protocol.create_server();
    let start_time = Instant::now();
    
    // Register Unison Protocol handlers
    register_unison_handlers(&mut server, start_time).await;
    
    info!("🎵 Unison Protocol Server Started!");
    info!("📡 Listening on: quic://127.0.0.1:8080 (QUIC Transport)");
    info!("🔧 Run client with: cargo run --example unison_ping_client");
    info!("📊 Available methods: ping, echo, get_server_time");
    info!("⏹️  Press Ctrl+C to stop");
    
    // Start the server
    server.listen("127.0.0.1:8080").await?;
    
    // Keep the server running
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        if !server.is_running() {
            break;
        }
    }
    
    Ok(())
}

async fn register_unison_handlers(server: &mut ProtocolServer, start_time: Instant) {
    // Register ping handler
    server.register_handler("ping", move |payload| {
        let request_time = Utc::now();
        
        let message = payload.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Hello from client!")
            .to_string();
            
        let sequence = payload.get("sequence")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        
        let expect_delay = payload.get("expect_delay")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        info!("🎵 Received ping: \"{}\" (seq: {}) at {}", 
              message, sequence, request_time.format("%H:%M:%S%.3f"));
        
        // Simulate delay if requested
        if expect_delay > 0 {
            std::thread::sleep(Duration::from_millis(expect_delay));
        }
        
        let response = json!({
            "message": format!("Pong: {}", message),
            "sequence": sequence,
            "server_info": "Unison Protocol Server v1.0.0",
            "processed_at": request_time.to_rfc3339()
        });
        
        info!("🎵 Sent pong: \"{}\" -> \"{}\"", 
              message, response["message"]);
        
        Ok(response) as Result<serde_json::Value, NetworkError>
    });
    
    // Register echo handler
    server.register_handler("echo", |payload| {
        let data = payload.get("data").cloned().unwrap_or_default();
        let transform = payload.get("transform")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        info!("🔄 Echo request with transform: '{}'", transform);
        
        let echoed_data = match transform {
            "uppercase" if data.is_string() => {
                json!(data.as_str().unwrap().to_uppercase())
            },
            "reverse" if data.is_string() => {
                json!(data.as_str().unwrap().chars().rev().collect::<String>())
            },
            _ => data.clone()
        };
        
        let response = json!({
            "echoed_data": echoed_data,
            "transformation_applied": if transform.is_empty() { None } else { Some(transform) }
        });
        
        Ok(response) as Result<serde_json::Value, NetworkError>
    });
    
    // Register get_server_time handler  
    server.register_handler("get_server_time", move |_payload| {
        let start = start_time;
        let now = Utc::now();
        let uptime_seconds = start.elapsed().as_secs();
        
        info!("⏰ Server time requested, uptime: {}s", uptime_seconds);
        
        let response = json!({
            "server_time": now.to_rfc3339(),
            "timezone": "UTC",
            "uptime_seconds": uptime_seconds
        });
        
        Ok(response) as Result<serde_json::Value, NetworkError>
    });
    
    info!("🎵 Unison Protocol handlers registered successfully");
}