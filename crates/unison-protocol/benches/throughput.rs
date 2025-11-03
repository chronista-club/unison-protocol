use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::runtime::Runtime;
use unison_protocol::network::{
    NetworkError, UnisonClient, UnisonServer, UnisonServerExt, quic::QuicClient,
};
use unison_protocol::{ProtocolClient, ProtocolServer};

/// バッチサイズのバリエーション
const BATCH_SIZES: &[u64] = &[1, 10, 100, 1000];

/// メッセージペイロードサイズ
const PAYLOAD_SIZES: &[usize] = &[128, 512, 2048, 8192];

/// メッセージ処理のスループット測定
fn bench_message_throughput(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mut group = c.benchmark_group("message_throughput");

    for &payload_size in PAYLOAD_SIZES {
        for &batch_size in BATCH_SIZES {
            let bench_name = format!("payload_{}_batch_{}", payload_size, batch_size);

            group.throughput(Throughput::Elements(batch_size));
            group.bench_function(bench_name, |b| {
                b.to_async(&runtime).iter(|| async move {
                    // サーバー起動
                    let processed = Arc::new(AtomicU64::new(0));
                    let processed_clone = processed.clone();

                    tokio::spawn(async move {
                        let mut server = ProtocolServer::new();
                        server.register_handler("process", move |payload| {
                            processed_clone.fetch_add(1, Ordering::Relaxed);
                            Ok(json!({
                                "status": "processed",
                                "id": payload.get("id").cloned().unwrap_or(json!(0))
                            }))
                                as Result<serde_json::Value, NetworkError>
                        });

                        let _ = server.listen("127.0.0.1:8081").await;
                        tokio::time::sleep(Duration::from_secs(3600)).await;
                    });

                    tokio::time::sleep(Duration::from_millis(100)).await;

                    // クライアント接続
                    let quic_client = QuicClient::new().unwrap();
                    let mut client = ProtocolClient::new(quic_client);
                    client.connect("127.0.0.1:8081").await.unwrap();

                    // バッチ送信
                    let payload = "x".repeat(payload_size);
                    for i in 0..batch_size {
                        let _: Result<serde_json::Value, _> = client
                            .call(
                                "process",
                                json!({
                                    "id": i,
                                    "data": payload.clone()
                                }),
                            )
                            .await;
                    }

                    client.disconnect().await.unwrap();

                    black_box(processed.load(Ordering::Relaxed))
                });
            });
        }
    }

    group.finish();
}

/// ストリーミングスループット測定
fn bench_streaming_throughput(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mut group = c.benchmark_group("streaming_throughput");
    group.measurement_time(Duration::from_secs(10));

    for &payload_size in PAYLOAD_SIZES {
        group.throughput(Throughput::Bytes(payload_size as u64));
        group.bench_function(format!("stream_{}_bytes", payload_size), |b| {
            b.to_async(&runtime).iter(|| async move {
                // サーバー起動
                tokio::spawn(async move {
                    let mut server = ProtocolServer::new();
                    server.register_handler("stream", |_| {
                        Ok(json!({"status": "received"})) as Result<serde_json::Value, NetworkError>
                    });

                    let _ = server.listen("127.0.0.1:8082").await;
                    tokio::time::sleep(Duration::from_secs(3600)).await;
                });

                tokio::time::sleep(Duration::from_millis(100)).await;

                // クライアント接続
                let quic_client = QuicClient::new().unwrap();
                let mut client = ProtocolClient::new(quic_client);
                client.connect("127.0.0.1:8082").await.unwrap();

                // ストリーミング送信
                let payload = "x".repeat(payload_size);
                let start = std::time::Instant::now();
                let mut bytes_sent = 0u64;

                while start.elapsed() < Duration::from_secs(1) {
                    if client
                        .call(
                            "stream",
                            json!({
                                "data": payload.clone()
                            }),
                        )
                        .await
                        .is_ok()
                    {
                        bytes_sent += payload_size as u64;
                    }
                }

                client.disconnect().await.unwrap();

                black_box(bytes_sent)
            });
        });
    }

    group.finish();
}

/// 並列処理のスループット測定
fn bench_parallel_throughput(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mut group = c.benchmark_group("parallel_throughput");
    group.measurement_time(Duration::from_secs(15));

    for &num_workers in &[1, 2, 4, 8, 16] {
        group.bench_function(format!("workers_{}", num_workers), |b| {
            b.to_async(&runtime).iter(|| async move {
                // サーバー起動
                let counter = Arc::new(AtomicU64::new(0));
                let counter_clone = counter.clone();

                tokio::spawn(async move {
                    let mut server = ProtocolServer::new();
                    server.register_handler("work", move |_| {
                        counter_clone.fetch_add(1, Ordering::Relaxed);
                        Ok(json!({"status": "done"})) as Result<serde_json::Value, NetworkError>
                    });

                    let _ = server.listen("127.0.0.1:8083").await;
                    tokio::time::sleep(Duration::from_secs(3600)).await;
                });

                tokio::time::sleep(Duration::from_millis(100)).await;

                // 並列クライアント
                let mut handles = vec![];
                for _ in 0..num_workers {
                    let handle = tokio::spawn(async move {
                        let quic_client = QuicClient::new().unwrap();
                        let mut client = ProtocolClient::new(quic_client);
                        client.connect("127.0.0.1:8083").await.unwrap();

                        let mut local_count = 0u64;
                        let start = std::time::Instant::now();

                        while start.elapsed() < Duration::from_secs(1) {
                            if client.call("work", json!({})).await.is_ok() {
                                local_count += 1;
                            }
                        }

                        client.disconnect().await.unwrap();
                        local_count
                    });
                    handles.push(handle);
                }

                // 結果集計
                let mut total = 0u64;
                for handle in handles {
                    total += handle.await.unwrap();
                }

                black_box(total)
            });
        });
    }

    group.finish();
}

/// バースト処理のスループット測定
fn bench_burst_throughput(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mut group = c.benchmark_group("burst_throughput");

    for &burst_size in &[10, 50, 100, 500, 1000] {
        group.throughput(Throughput::Elements(burst_size));
        group.bench_function(format!("burst_{}", burst_size), |b| {
            b.to_async(&runtime).iter(|| async move {
                // サーバー起動
                tokio::spawn(async move {
                    let mut server = ProtocolServer::new();
                    server.register_handler("burst", |payload| {
                        Ok(payload) as Result<serde_json::Value, NetworkError>
                    });

                    let _ = server.listen("127.0.0.1:8084").await;
                    tokio::time::sleep(Duration::from_secs(3600)).await;
                });

                tokio::time::sleep(Duration::from_millis(100)).await;

                // クライアント接続
                let quic_client = QuicClient::new().unwrap();
                let mut client = ProtocolClient::new(quic_client);
                client.connect("127.0.0.1:8084").await.unwrap();

                // バースト送信
                let start = std::time::Instant::now();
                let mut success_count = 0;

                for i in 0..burst_size {
                    let result = client
                        .call(
                            "burst",
                            json!({
                                "id": i,
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            }),
                        )
                        .await;
                    if result.is_ok() {
                        success_count += 1;
                    }
                }

                let elapsed = start.elapsed();

                client.disconnect().await.unwrap();

                black_box((success_count, elapsed))
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_message_throughput,
    bench_streaming_throughput,
    bench_parallel_throughput,
    bench_burst_throughput
);

criterion_main!(benches);
