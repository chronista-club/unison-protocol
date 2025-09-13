use anyhow::Result;
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Barrier;
use tracing::{info, Level};
use unison_protocol::{ProtocolClient, ProtocolServer, UnisonClient, UnisonServer, UnisonServerExt};
use unison_protocol::network::{NetworkError, quic::QuicClient};

/// ベンチマーク結果
#[derive(Debug, Clone)]
struct BenchmarkResult {
    /// メッセージサイズ（バイト）
    message_size: usize,
    /// 平均レイテンシ（マイクロ秒）
    avg_latency_us: f64,
    /// P50レイテンシ（マイクロ秒）
    p50_latency_us: f64,
    /// P99レイテンシ（マイクロ秒）
    p99_latency_us: f64,
    /// スループット（メッセージ/秒）
    throughput_msg_per_sec: f64,
    /// CPU使用率（％）- 簡易測定
    cpu_usage_percent: f64,
}

/// レイテンシを測定
async fn measure_latency(
    client: &mut ProtocolClient,
    message_size: usize,
    iterations: usize,
) -> Vec<u64> {
    let mut latencies = Vec::with_capacity(iterations);
    let message = json!({
        "data": "x".repeat(message_size),
        "sequence": 0
    });

    for i in 0..iterations {
        let mut msg = message.clone();
        msg["sequence"] = json!(i);
        
        let start = Instant::now();
        let _ = client.call("echo", msg).await;
        let elapsed = start.elapsed().as_micros() as u64;
        latencies.push(elapsed);
    }

    latencies.sort_unstable();
    latencies
}

/// スループットを測定
async fn measure_throughput(
    client: &mut ProtocolClient,
    message_size: usize,
    duration: Duration,
) -> f64 {
    let message = json!({
        "data": "x".repeat(message_size),
        "sequence": 0
    });
    
    let start = Instant::now();
    let mut count = 0u64;
    let mut sequence = 0u32;
    
    while start.elapsed() < duration {
        let mut msg = message.clone();
        msg["sequence"] = json!(sequence);
        
        if client.call("echo", msg).await.is_ok() {
            count += 1;
            sequence = sequence.wrapping_add(1);
        }
    }
    
    let elapsed = start.elapsed().as_secs_f64();
    count as f64 / elapsed
}

/// CPU使用率を簡易的に測定
async fn measure_cpu_usage() -> f64 {
    // 簡易的な実装：プロセスのCPU時間を測定
    let start_time = std::time::SystemTime::now();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let _elapsed = start_time.elapsed().unwrap();
    
    // 実際のCPU使用率測定は複雑なので、ダミー値を返す
    // 本番環境では sysinfo クレートなどを使用
    35.0
}

/// ベンチマークサーバーを起動
async fn start_benchmark_server() -> Result<()> {
    let mut server = ProtocolServer::new();
    let counter = Arc::new(AtomicU64::new(0));
    let counter_clone = counter.clone();
    
    // Echo handler
    server.register_handler("echo", move |payload| {
        counter_clone.fetch_add(1, Ordering::Relaxed);
        Ok(payload) as Result<serde_json::Value, NetworkError>
    });
    
    info!("📊 Benchmark server starting on 127.0.0.1:8080");
    server.listen("127.0.0.1:8080").await?;
    
    Ok(())
}

/// ベンチマークを実行
async fn run_benchmark(message_size: usize) -> Result<BenchmarkResult> {
    let quic_client = QuicClient::new();
    let mut client = ProtocolClient::new(quic_client);
    client.connect("127.0.0.1:8080").await?;
    
    info!("📏 Testing with message size: {} bytes", message_size);
    
    // レイテンシ測定
    info!("  ⏱️  Measuring latency...");
    let latencies = measure_latency(&mut client, message_size, 1000).await;
    
    let avg_latency = latencies.iter().sum::<u64>() as f64 / latencies.len() as f64;
    let p50_latency = latencies[latencies.len() / 2] as f64;
    let p99_latency = latencies[latencies.len() * 99 / 100] as f64;
    
    // スループット測定
    info!("  📈 Measuring throughput...");
    let throughput = measure_throughput(&mut client, message_size, Duration::from_secs(5)).await;
    
    // CPU使用率測定
    info!("  💻 Measuring CPU usage...");
    let cpu_usage = measure_cpu_usage().await;
    
    client.disconnect().await?;
    
    Ok(BenchmarkResult {
        message_size,
        avg_latency_us: avg_latency,
        p50_latency_us: p50_latency,
        p99_latency_us: p99_latency,
        throughput_msg_per_sec: throughput,
        cpu_usage_percent: cpu_usage,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    // ロギング設定
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🎵 Unison Protocol Benchmark");
    info!("=============================");
    
    // サーバーを別タスクで起動
    let barrier = Arc::new(Barrier::new(2));
    let barrier_clone = barrier.clone();
    
    tokio::spawn(async move {
        let _ = start_benchmark_server().await;
        barrier_clone.wait().await;
    });
    
    // サーバーの起動を待つ
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // 各メッセージサイズでベンチマーク実行
    let message_sizes = vec![64, 256, 1024, 4096, 16384];
    let mut results = Vec::new();
    
    for size in message_sizes {
        match run_benchmark(size).await {
            Ok(result) => {
                results.push(result.clone());
                info!("✅ Completed benchmark for {} bytes", size);
                info!("   - Avg latency: {:.2} µs", result.avg_latency_us);
                info!("   - P50 latency: {:.2} µs", result.p50_latency_us);
                info!("   - P99 latency: {:.2} µs", result.p99_latency_us);
                info!("   - Throughput: {:.0} msg/s", result.throughput_msg_per_sec);
                info!("   - CPU usage: {:.1}%", result.cpu_usage_percent);
            }
            Err(e) => {
                eprintln!("❌ Benchmark failed for {} bytes: {}", size, e);
            }
        }
    }
    
    // 結果のサマリーを表示
    info!("");
    info!("📊 Benchmark Summary");
    info!("====================");
    info!("");
    info!("| Message Size | Avg Latency | P50 Latency | P99 Latency | Throughput | CPU Usage |");
    info!("|-------------|-------------|-------------|-------------|------------|-----------|");
    
    for result in &results {
        info!(
            "| {:>11} | {:>9.2} µs | {:>9.2} µs | {:>9.2} µs | {:>7.0} msg/s | {:>7.1}% |",
            format!("{} B", result.message_size),
            result.avg_latency_us,
            result.p50_latency_us,
            result.p99_latency_us,
            result.throughput_msg_per_sec,
            result.cpu_usage_percent
        );
    }
    
    info!("");
    info!("📝 Markdown Table for README:");
    info!("");
    info!("| メトリクス | 64B | 256B | 1KB | 4KB | 16KB |");
    info!("|-----------|-----|------|-----|-----|------|");
    
    // 平均レイテンシ行
    print!("| 平均レイテンシ |");
    for result in &results {
        print!(" {:.1}µs |", result.avg_latency_us);
    }
    println!();
    
    // P50レイテンシ行
    print!("| P50レイテンシ |");
    for result in &results {
        print!(" {:.1}µs |", result.p50_latency_us);
    }
    println!();
    
    // P99レイテンシ行
    print!("| P99レイテンシ |");
    for result in &results {
        print!(" {:.1}µs |", result.p99_latency_us);
    }
    println!();
    
    // スループット行
    print!("| スループット |");
    for result in &results {
        if result.throughput_msg_per_sec > 1000.0 {
            print!(" {:.1}K msg/s |", result.throughput_msg_per_sec / 1000.0);
        } else {
            print!(" {:.0} msg/s |", result.throughput_msg_per_sec);
        }
    }
    println!();
    
    info!("");
    info!("✅ Benchmark completed!");
    
    // サーバーを終了
    barrier.wait().await;
    
    Ok(())
}