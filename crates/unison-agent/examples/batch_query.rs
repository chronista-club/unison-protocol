//! バッチクエリの例

use unison_agent::AgentClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ロギングの初期化
    tracing_subscriber::fmt()
        .with_env_filter("unison_agent=info")
        .init();

    println!("=== Unison Agent: Batch Query Example ===\n");

    // エージェントクライアントの作成
    let client = AgentClient::new();

    // 複数のクエリを準備
    let prompts = vec![
        "What is the capital of Japan?",
        "What is 10 + 15?",
        "Name one programming language that starts with 'R'.",
    ];

    println!("Executing {} queries in batch...\n", prompts.len());

    // バッチクエリを実行
    let responses = client.batch_query(prompts.clone()).await?;

    // 結果を表示
    for (i, (prompt, response)) in prompts.iter().zip(responses.iter()).enumerate() {
        println!("--- Query {} ---", i + 1);
        println!("Prompt: {}", prompt);
        println!("Response: {}\n", response);
    }

    println!("All queries completed!");

    Ok(())
}
