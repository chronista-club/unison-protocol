use anyhow::Result;
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{info, Level};
use unison_protocol::{UnisonProtocol, ProtocolServer, ProtocolClient};
use unison_protocol::network::{NetworkError, UnisonServer, UnisonClient, UnisonServerExt};

/// QUIC統合テスト - サーバーとクライアントを同一プロセスでテスト
#[tokio::test]
async fn test_quic_server_client_integration() -> Result<()> {
    // ログ初期化
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_test_writer()
        .init();

    info!("🧪 Starting QUIC integration test");

    // サーバーとクライアントを同時に実行
    let server_handle = tokio::spawn(run_test_server());
    let client_handle = tokio::spawn(run_test_client());

    // サーバーが起動するまで少し待機
    tokio::time::sleep(Duration::from_millis(500)).await;

    // クライアントテストが完了するまで待機（タイムアウト付き）
    let client_result = timeout(Duration::from_secs(30), client_handle).await.map_err(|_| anyhow::anyhow!("Test timeout"))??;
    
    // サーバーを停止
    server_handle.abort();

    info!("🎉 QUIC integration test completed successfully");
    client_result
}

/// テスト用サーバーの実行
async fn run_test_server() -> Result<()> {
    info!("🎵 Starting test server...");
    
    // Unison protocolインスタンス作成
    let mut protocol = UnisonProtocol::new();
    protocol.load_schema(include_str!("../schemas/ping_pong.kdl"))?;
    
    // サーバー作成とハンドラー登録
    let mut server = protocol.create_server();
    let start_time = Instant::now();
    
    register_test_handlers(&mut server, start_time).await;
    
    info!("🎵 Test server started on 127.0.0.1:8080");
    
    // サーバー開始（無限ループ）
    server.listen("127.0.0.1:8080").await?;
    
    Ok(())
}

/// テスト用クライアントの実行
async fn run_test_client() -> Result<()> {
    info!("🔌 Starting test client...");
    
    // サーバーが完全に起動するまで待機
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Unison protocolインスタンス作成
    let mut protocol = UnisonProtocol::new();
    protocol.load_schema(include_str!("../schemas/ping_pong.kdl"))?;
    
    // クライアント作成と接続
    let mut client = protocol.create_client()?;
    client.connect("127.0.0.1:8080").await?;
    info!("✅ Connected to test server");
    
    // テストケース実行
    run_integration_tests(&mut client).await?;
    
    // 切断
    client.disconnect().await?;
    info!("👋 Disconnected from test server");
    
    Ok(())
}

/// テスト用ハンドラーの登録
async fn register_test_handlers(server: &mut ProtocolServer, start_time: Instant) {
    // ping handler
    server.register_handler("ping", move |payload| {
        let message = payload.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Hello!")
            .to_string();
            
        let sequence = payload.get("sequence")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        
        let response = json!({
            "message": format!("Pong: {}", message),
            "sequence": sequence,
            "server_info": "Test Server v1.0.0"
        });
        
        Ok(response) as Result<serde_json::Value, NetworkError>
    });
    
    // echo handler
    server.register_handler("echo", |payload| {
        let data = payload.get("data").cloned().unwrap_or_default();
        let transform = payload.get("transform")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
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
    
    // get_server_time handler
    server.register_handler("get_server_time", move |_payload| {
        let start = start_time;
        let uptime_seconds = start.elapsed().as_secs();
        
        let response = json!({
            "server_time": chrono::Utc::now().to_rfc3339(),
            "timezone": "UTC",
            "uptime_seconds": uptime_seconds
        });
        
        Ok(response) as Result<serde_json::Value, NetworkError>
    });
    
    info!("🎵 Test handlers registered");
    
    // Wait for handlers to be fully registered
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
}

/// 統合テストの実行
async fn run_integration_tests(client: &mut ProtocolClient) -> Result<()> {
    info!("🧪 Running integration tests...");
    
    // Test 1: Server time check
    info!("Test 1: Server time check");
    let response = client.call("get_server_time", json!({})).await?;
    info!("📋 Server response: {}", serde_json::to_string_pretty(&response)?);
    assert!(response.get("server_time").is_some(), "Server time should be present");
    assert!(response.get("uptime_seconds").is_some(), "Uptime should be present");
    info!("✅ Server time test passed");
    
    // Test 2: Basic ping-pong
    info!("Test 2: Basic ping-pong");
    for i in 1..=3 {
        let response = client.call("ping", json!({
            "message": format!("Test message {}", i),
            "sequence": i
        })).await?;
        
        let pong_message = response.get("message")
            .and_then(|v| v.as_str())
            .expect("Pong message should be present");
        
        assert!(pong_message.contains("Pong:"), "Response should contain 'Pong:'");
        assert_eq!(
            response.get("sequence").and_then(|v| v.as_i64()).unwrap(),
            i,
            "Sequence number should match"
        );
    }
    info!("✅ Ping-pong test passed");
    
    // Test 3: Echo with transformations
    info!("Test 3: Echo transformations");
    
    // Uppercase test
    let response = client.call("echo", json!({
        "data": "hello world",
        "transform": "uppercase"
    })).await?;
    assert_eq!(
        response.get("echoed_data").and_then(|v| v.as_str()).unwrap(),
        "HELLO WORLD",
        "Uppercase transformation should work"
    );
    
    // Reverse test
    let response = client.call("echo", json!({
        "data": "abcd",
        "transform": "reverse"
    })).await?;
    assert_eq!(
        response.get("echoed_data").and_then(|v| v.as_str()).unwrap(),
        "dcba",
        "Reverse transformation should work"
    );
    
    // No transformation test
    let response = client.call("echo", json!({
        "data": "unchanged",
        "transform": ""
    })).await?;
    assert_eq!(
        response.get("echoed_data").and_then(|v| v.as_str()).unwrap(),
        "unchanged",
        "No transformation should leave data unchanged"
    );
    info!("✅ Echo transformation tests passed");
    
    // Test 4: Performance test (reduced size for integration test)
    info!("Test 4: Performance test");
    let start_time = Instant::now();
    
    for i in 1..=10 {  // Reduced from 20 to 10 for faster test execution
        let _response = client.call("ping", json!({
            "message": format!("Perf test {}", i),
            "sequence": i + 100
        })).await?;
    }
    
    let elapsed = start_time.elapsed();
    info!("✅ Performance test completed in {:?}", elapsed);
    
    // Test 5: Complex JSON handling
    info!("Test 5: Complex JSON handling");
    let complex_data = json!({
        "nested": {
            "array": [1, 2, 3],
            "string": "test",
            "boolean": true
        },
        "number": 42
    });
    
    let response = client.call("echo", json!({
        "data": complex_data.clone(),
        "transform": ""
    })).await?;
    
    let echoed = response.get("echoed_data").unwrap();
    assert_eq!(
        echoed.get("nested").unwrap().get("array").unwrap(),
        &json!([1, 2, 3]),
        "Complex nested data should be preserved"
    );
    info!("✅ Complex JSON test passed");
    
    info!("🎉 All integration tests passed!");
    Ok(())
}

/// rust-embed証明書の使用テスト
#[tokio::test]
async fn test_rust_embed_certificates() -> Result<()> {
    info!("🔐 Testing rust-embed certificate loading");
    
    // QuicServerのcert loading methodを直接テスト
    use unison_protocol::network::quic::QuicServer;
    
    let result = QuicServer::load_cert_embedded();
    match result {
        Ok((certs, _private_key)) => {
            info!("✅ rust-embed certificates loaded successfully");
            assert!(!certs.is_empty(), "Certificate chain should not be empty");
            info!("📜 Certificate count: {}", certs.len());
        },
        Err(e) => {
            info!("ℹ️  rust-embed certificates not found (expected in some environments): {}", e);
            // フォールバックテスト
            let result = QuicServer::load_cert_auto();
            assert!(result.is_ok(), "Auto certificate loading should work");
            info!("✅ Auto certificate loading works");
        }
    }
    
    Ok(())
}

/// QUIC設定の検証テスト
#[tokio::test]
async fn test_quic_configuration() -> Result<()> {
    info!("🔧 Testing QUIC configuration");
    
    use unison_protocol::network::quic::{QuicServer, QuicClient};
    
    // Server configuration test
    let server_config = QuicServer::configure_server().await;
    assert!(server_config.is_ok(), "Server configuration should be valid");
    info!("✅ Server configuration test passed");
    
    // Client configuration test
    let client_config = QuicClient::configure_client().await;
    assert!(client_config.is_ok(), "Client configuration should be valid");
    info!("✅ Client configuration test passed");
    
    Ok(())
}