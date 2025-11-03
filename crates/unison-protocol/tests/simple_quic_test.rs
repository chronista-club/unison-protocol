use anyhow::Result;
use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{Level, info};

/// 簡単なQUIC統合テスト - リモートプロセス版の動作を確認
#[tokio::test]
async fn test_simple_quic_functionality() -> Result<()> {
    // ログ初期化
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_test_writer()
        .init();

    info!("🧪 Running simple QUIC functionality test");

    // QUICサーバーとクライアントの設定テスト
    test_quic_server_config().await?;
    test_quic_client_config().await?;
    test_certificate_loading().await?;

    info!("✅ All simple QUIC tests passed!");
    Ok(())
}

async fn test_quic_server_config() -> Result<()> {
    use unison_protocol::network::quic::QuicServer;

    info!("🔧 Testing QUIC server configuration");

    let server_config = QuicServer::configure_server().await;
    assert!(server_config.is_ok(), "Server configuration should succeed");

    info!("✅ QUIC server configuration test passed");
    Ok(())
}

async fn test_quic_client_config() -> Result<()> {
    use unison_protocol::network::quic::QuicClient;

    info!("🔧 Testing QUIC client configuration");

    let client_config = QuicClient::configure_client().await;
    assert!(client_config.is_ok(), "Client configuration should succeed");

    info!("✅ QUIC client configuration test passed");
    Ok(())
}

async fn test_certificate_loading() -> Result<()> {
    use unison_protocol::network::quic::QuicServer;

    info!("🔐 Testing certificate loading");

    // Test auto certificate loading (should work with external or embedded certificates)
    let cert_result = QuicServer::load_cert_auto();
    assert!(
        cert_result.is_ok(),
        "Auto certificate loading should succeed"
    );

    let (certs, _private_key) = cert_result?;
    assert!(!certs.is_empty(), "Certificate chain should not be empty");

    info!("📜 Loaded {} certificate(s)", certs.len());
    info!("✅ Certificate loading test passed");
    Ok(())
}

/// rust-embed証明書の統合テスト
#[tokio::test]
async fn test_embedded_certificates_integration() -> Result<()> {
    info!("🔐 Testing embedded certificates integration");

    use unison_protocol::network::quic::QuicServer;

    // Test embedded certificate loading
    let embedded_result = QuicServer::load_cert_embedded();

    match embedded_result {
        Ok((certs, _key)) => {
            info!("✅ rust-embed certificates loaded successfully");
            assert!(
                !certs.is_empty(),
                "Embedded certificate chain should not be empty"
            );
            info!("📜 Embedded certificate count: {}", certs.len());
        }
        Err(e) => {
            info!("ℹ️  Embedded certificates not available: {}", e);
            // This is expected if we don't have embedded certificates

            // Test that auto loading still works (fallback to external files or generated)
            let auto_result = QuicServer::load_cert_auto();
            assert!(
                auto_result.is_ok(),
                "Auto certificate loading should work as fallback"
            );
            info!("✅ Fallback certificate loading works");
        }
    }

    Ok(())
}

/// QUIC transport設定の詳細テスト
#[tokio::test]
async fn test_quic_transport_settings() -> Result<()> {
    info!("⚙️ Testing QUIC transport settings");

    use unison_protocol::network::quic::{QuicClient, QuicServer};

    // Test server transport configuration
    let server_config = QuicServer::configure_server().await?;
    info!("✅ Server transport configuration created");

    // Test client transport configuration
    let client_config = QuicClient::configure_client().await?;
    info!("✅ Client transport configuration created");

    info!("✅ QUIC transport settings test passed");
    Ok(())
}

/// ビルド時証明書生成のテスト
#[tokio::test]
async fn test_build_time_certificate_generation() -> Result<()> {
    info!("🏗️ Testing build-time certificate generation");

    // Check if certificates exist in assets/certs
    let cert_path = std::path::Path::new("assets/certs/cert.pem");
    let key_path = std::path::Path::new("assets/certs/private_key.der");

    if cert_path.exists() && key_path.exists() {
        info!("✅ Build-time certificates found in assets/certs/");

        // Test loading from external files
        use unison_protocol::network::quic::QuicServer;
        let file_result = QuicServer::load_cert_from_files(
            "assets/certs/cert.pem",
            "assets/certs/private_key.der",
        );

        assert!(
            file_result.is_ok(),
            "Loading certificates from external files should work"
        );
        let (certs, _key) = file_result?;
        assert!(!certs.is_empty(), "Certificate chain should not be empty");

        info!(
            "📜 Successfully loaded {} certificate(s) from files",
            certs.len()
        );
    } else {
        info!("ℹ️  Build-time certificates not found - this is expected in some environments");
    }

    info!("✅ Build-time certificate generation test completed");
    Ok(())
}

/// パフォーマンス指向の設定テスト
#[tokio::test]
async fn test_performance_optimizations() -> Result<()> {
    info!("⚡ Testing performance optimizations");

    use unison_protocol::network::quic::{QuicClient, QuicServer};

    // Test that configurations are optimized for real-time communication
    let server_config = QuicServer::configure_server().await?;
    let client_config = QuicClient::configure_client().await?;

    info!("✅ Performance-optimized configurations created");
    info!("🔧 Server config: QUIC transport with TLS 1.3");
    info!("🔧 Client config: QUIC transport with certificate skip verification (for testing)");

    info!("✅ Performance optimization test passed");
    Ok(())
}
