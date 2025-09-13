use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use unison_protocol::network::{ProtocolClient, ProtocolServer, ProtocolClientTrait, ProtocolServerTrait};
use serde_json::json;
use std::time::Duration;
use tokio::runtime::Runtime;
use std::sync::Arc;
use tokio::sync::Barrier;
use hdrhistogram::Histogram;

/// メッセージサイズのバリエーション
const MESSAGE_SIZES: &[usize] = &[64, 256, 1024, 4096, 16384];

/// レイテンシ測定用の関数
async fn measure_latency(
    client: &mut ProtocolClient,
    message_size: usize,
    iterations: u32,
) -> Histogram<u64> {
    let mut histogram = Histogram::<u64>::new(3).unwrap();
    let message = json!({
        "data": "x".repeat(message_size),
        "sequence": 0
    });

    for i in 0..iterations {
        let start = std::time::Instant::now();
        let mut msg = message.clone();
        msg["sequence"] = json!(i);
        
        let _ = client.call("echo", msg).await;
        let elapsed = start.elapsed().as_micros() as u64;
        histogram.record(elapsed).unwrap();
    }

    histogram
}

/// スループット測定用の関数
async fn measure_throughput(
    client: &mut ProtocolClient,
    message_size: usize,
    duration_secs: u64,
) -> f64 {
    let message = json!({
        "data": "x".repeat(message_size),
        "sequence": 0
    });
    
    let start = std::time::Instant::now();
    let mut count = 0u64;
    let mut sequence = 0u32;
    
    while start.elapsed().as_secs() < duration_secs {
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

/// サーバーのセットアップ
async fn setup_server() -> Arc<Barrier> {
    let barrier = Arc::new(Barrier::new(2));
    let barrier_clone = barrier.clone();
    
    tokio::spawn(async move {
        let mut server = ProtocolServer::new();
        
        // Echo handler
        server.register_handler("echo", |payload| {
            Ok(payload)
        });
        
        // サーバー起動
        let _ = server.listen("127.0.0.1:0").await;
        
        // バリアで同期
        barrier_clone.wait().await;
        
        // サーバーを維持
        tokio::time::sleep(Duration::from_secs(3600)).await;
    });
    
    // サーバーの起動を待つ
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    barrier
}

fn bench_latency(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("quic_latency");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);
    
    for &size in MESSAGE_SIZES {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &size,
            |b, &size| {
                b.to_async(&runtime).iter(|| async move {
                    let barrier = setup_server().await;
                    let mut client = ProtocolClient::new();
                    client.connect("127.0.0.1:8080").await.unwrap();
                    
                    let histogram = measure_latency(&mut client, size, 100).await;
                    
                    // バリアで同期してサーバーを終了
                    barrier.wait().await;
                    
                    black_box(histogram.mean())
                });
            },
        );
    }
    
    group.finish();
}

fn bench_throughput(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("quic_throughput");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);
    
    for &size in MESSAGE_SIZES {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &size,
            |b, &size| {
                b.to_async(&runtime).iter(|| async move {
                    let barrier = setup_server().await;
                    let mut client = ProtocolClient::new();
                    client.connect("127.0.0.1:8080").await.unwrap();
                    
                    let throughput = measure_throughput(&mut client, size, 5).await;
                    
                    // バリアで同期してサーバーを終了
                    barrier.wait().await;
                    
                    black_box(throughput)
                });
            },
        );
    }
    
    group.finish();
}

fn bench_connection_establishment(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();
    
    c.bench_function("quic_connection_establishment", |b| {
        b.to_async(&runtime).iter(|| async {
            let barrier = setup_server().await;
            
            let start = std::time::Instant::now();
            let mut client = ProtocolClient::new();
            client.connect("127.0.0.1:8080").await.unwrap();
            let elapsed = start.elapsed();
            
            client.disconnect().await.unwrap();
            barrier.wait().await;
            
            black_box(elapsed)
        });
    });
}

fn bench_concurrent_connections(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("quic_concurrent_connections");
    
    for &num_clients in &[1, 5, 10, 20, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_clients),
            &num_clients,
            |b, &num_clients| {
                b.to_async(&runtime).iter(|| async move {
                    let barrier = setup_server().await;
                    let client_barrier = Arc::new(Barrier::new(num_clients + 1));
                    
                    let mut handles = vec![];
                    
                    for _ in 0..num_clients {
                        let client_barrier_clone = client_barrier.clone();
                        let handle = tokio::spawn(async move {
                            let mut client = ProtocolClient::new();
                            client.connect("127.0.0.1:8080").await.unwrap();
                            
                            // 全クライアントが接続するまで待つ
                            client_barrier_clone.wait().await;
                            
                            // 100回のリクエストを送信
                            for i in 0..100 {
                                let _ = client.call("echo", json!({
                                    "data": "test",
                                    "sequence": i
                                })).await;
                            }
                            
                            client.disconnect().await.unwrap();
                        });
                        handles.push(handle);
                    }
                    
                    // 全クライアントを開始
                    client_barrier.wait().await;
                    
                    // 全クライアントの完了を待つ
                    for handle in handles {
                        handle.await.unwrap();
                    }
                    
                    barrier.wait().await;
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_latency,
    bench_throughput,
    bench_connection_establishment,
    bench_concurrent_connections
);

criterion_main!(benches);