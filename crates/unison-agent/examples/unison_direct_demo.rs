//! Unison Protocol直接統合デモ
//!
//! このExampleは、UnisonToolsを直接使用して
//! Unisonサービスにアクセスする方法を示します。

use serde_json::json;
use unison_agent::UnisonTools;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ロギングの初期化
    tracing_subscriber::fmt()
        .with_env_filter("unison_agent=debug")
        .init();

    println!("=== Unison Agent: Direct Integration Demo ===\n");

    // UnisonToolsインスタンスを作成
    let mut tools = UnisonTools::new();

    // Demo 1: 接続状態の確認
    println!("--- Demo 1: Check connection status ---");
    let connected = tools.is_connected().await;
    println!("Connected: {}\n", connected);

    // Demo 2: サーバーへの接続（シミュレーション）
    println!("--- Demo 2: Connect to Unison server ---");
    println!("Attempting to connect to: https://[::1]:8080");

    match tools.connect("https://[::1]:8080").await {
        Ok(_) => {
            println!("✓ Successfully connected!\n");

            // Demo 3: サービス一覧の取得
            println!("--- Demo 3: List available services ---");
            match tools.list_services().await {
                Ok(services) => {
                    println!("Available services:");
                    for service in services {
                        println!("  - {}", service);
                    }
                    println!();
                }
                Err(e) => {
                    eprintln!("✗ Failed to list services: {}\n", e);
                }
            }

            // Demo 4: サービスの呼び出し
            println!("--- Demo 4: Call a service method ---");
            println!("Calling EchoService::ping");

            match tools
                .call_service(
                    "EchoService",
                    "ping",
                    json!({
                        "message": "Hello from Unison Agent!"
                    }),
                )
                .await
            {
                Ok(response) => {
                    println!("✓ Response received:");
                    println!("{}\n", serde_json::to_string_pretty(&response)?);
                }
                Err(e) => {
                    eprintln!("✗ Service call failed: {}\n", e);
                }
            }

            // Demo 5: 切断
            println!("--- Demo 5: Disconnect from server ---");
            match tools.disconnect().await {
                Ok(_) => {
                    println!("✓ Successfully disconnected!\n");
                }
                Err(e) => {
                    eprintln!("✗ Disconnect failed: {}\n", e);
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Connection failed: {}", e);
            eprintln!("\nNote: This is expected if no Unison server is running.");
            eprintln!("To run a real server, start one with:");
            eprintln!("  cargo run --example unison_ping_server\n");
        }
    }

    println!("=== Demo Completed ===\n");
    println!("This demo showed:");
    println!("1. Creating UnisonTools instance");
    println!("2. Checking connection status");
    println!("3. Connecting to a Unison server");
    println!("4. Listing available services");
    println!("5. Calling service methods");
    println!("6. Disconnecting from the server");
    println!("\nFor Claude Agent integration, see: unison_mcp_demo.rs");

    Ok(())
}
