use std::collections::HashMap;
use super::{SystemStream, NetworkError, StreamHandle};

/// Unisonサービストレイト - SystemStreamをベースとした高レベルサービスインターフェース
#[allow(async_fn_in_trait)]
pub trait Service: SystemStream {
    /// サービス種別識別子
    fn service_type(&self) -> &str;
    
    /// サービス名
    fn service_name(&self) -> &str;
    
    /// サービスメタデータと設定
    fn metadata(&self) -> HashMap<String, String>;
    
    /// サービスバージョン
    fn version(&self) -> &str { "1.0.0" }
    
    /// メタデータ付き構造化データの送信
    async fn send_with_metadata(
        &mut self, 
        data: serde_json::Value,
        metadata: HashMap<String, String>
    ) -> Result<(), NetworkError> {
        let wrapped_data = serde_json::json!({
            "data": data,
            "metadata": metadata,
            "service_type": self.service_type(),
            "service_name": self.service_name(),
            "service_version": self.version(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        self.send(wrapped_data).await
    }
    
    /// メタデータ抽出付き構造化データの受信
    async fn receive_with_metadata(&mut self) -> Result<(serde_json::Value, HashMap<String, String>), NetworkError> {
        let received = self.receive().await?;
        
        if let Some(obj) = received.as_object() {
            let data = obj.get("data").cloned().unwrap_or_default();
            let metadata = obj.get("metadata")
                .and_then(|m| m.as_object())
                .map(|m| m.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect())
                .unwrap_or_default();
            
            Ok((data, metadata))
        } else {
            // 生データのフォールバック
            Ok((received, HashMap::new()))
        }
    }
    
    /// ヘルスモニタリング用サービスハートビートの開始
    async fn start_service_heartbeat(&mut self, interval_secs: u64) -> Result<(), NetworkError> {
        let heartbeat_data = serde_json::json!({
            "type": "service_heartbeat",
            "service": self.service_name(),
            "version": self.version(),
            "interval": interval_secs,
            "started_at": chrono::Utc::now().to_rfc3339()
        });
        
        self.send_with_metadata(heartbeat_data, HashMap::from([
            ("message_type".to_string(), "service_heartbeat_start".to_string()),
            ("service_name".to_string(), self.service_name().to_string()),
        ])).await
    }
    
    /// サービスヘルスpingの送信
    async fn service_ping(&mut self) -> Result<(), NetworkError> {
        let ping_data = serde_json::json!({
            "type": "service_ping",
            "service": self.service_name(),
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        self.send_with_metadata(ping_data, HashMap::from([
            ("message_type".to_string(), "service_ping".to_string()),
            ("service_name".to_string(), self.service_name().to_string()),
        ])).await
    }
    
    /// サービスリクエストの処理
    async fn handle_request(
        &mut self, 
        method: &str, 
        payload: serde_json::Value
    ) -> Result<serde_json::Value, NetworkError>;
    
    /// サービス機能の取得
    fn get_capabilities(&self) -> Vec<String> {
        vec!["ping".to_string(), "heartbeat".to_string()]
    }
    
    /// サービス終了通知
    async fn shutdown(&mut self) -> Result<(), NetworkError> {
        let shutdown_data = serde_json::json!({
            "type": "service_shutdown",
            "service": self.service_name(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        self.send_with_metadata(shutdown_data, HashMap::from([
            ("message_type".to_string(), "service_shutdown".to_string()),
        ])).await?;
        
        self.close().await
    }
}

/// リアルタイム機能付きの拡張Service
#[allow(async_fn_in_trait)]
pub trait RealtimeService: Service {
    /// 優先度付き時間感応型サービスデータの送信
    async fn send_realtime(
        &mut self, 
        method: &str,
        data: serde_json::Value,
        priority: ServicePriority
    ) -> Result<(), NetworkError>;
    
    /// タイムアウト付きサービスデータの受信
    async fn receive_with_timeout(
        &mut self, 
        timeout: std::time::Duration
    ) -> Result<serde_json::Value, NetworkError>;
    
    /// サービスパフォーマンス統計の取得
    async fn get_performance_stats(&self) -> Result<ServiceStats, NetworkError>;
}

/// リアルタイム通信のためのサービス優先度レベル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServicePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// サービスパフォーマンス統計
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceStats {
    pub avg_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub packet_loss_rate: f64,
    pub jitter_ms: f64,
    pub requests_processed: u64,
    pub errors_count: u64,
    pub uptime_seconds: u64,
}

impl Default for ServiceStats {
    fn default() -> Self {
        Self {
            avg_latency_ms: 0.0,
            min_latency_ms: 0.0,
            max_latency_ms: 0.0,
            packet_loss_rate: 0.0,
            jitter_ms: 0.0,
            requests_processed: 0,
            errors_count: 0,
            uptime_seconds: 0,
        }
    }
}

/// 様々な用途に対応するサービス設定
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub service_name: String,
    pub service_version: String,
    pub buffer_size: usize,
    pub max_message_size: usize,
    pub heartbeat_interval: Option<std::time::Duration>,
    pub priority: ServicePriority,
    pub reliable_delivery: bool,
    pub ordered_delivery: bool,
    pub max_concurrent_requests: u32,
    pub request_timeout: std::time::Duration,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            service_name: "unison-service".to_string(),
            service_version: "1.0.0".to_string(),
            buffer_size: 1024 * 1024, // 1MB
            max_message_size: 8 * 1024 * 1024, // 8MB
            heartbeat_interval: Some(std::time::Duration::from_secs(30)),
            priority: ServicePriority::Normal,
            reliable_delivery: true, // QUIC is reliable by default
            ordered_delivery: true,  // QUIC maintains order within streams
            max_concurrent_requests: 100,
            request_timeout: std::time::Duration::from_secs(30),
        }
    }
}

/// Unison Service implementation with QUIC SystemStream
pub struct UnisonService {
    config: ServiceConfig,
    stream: Box<crate::network::quic::UnisonStream>,
    stats: ServiceStats,
    start_time: std::time::Instant,
}

impl UnisonService {
    pub fn new(config: ServiceConfig, stream: crate::network::quic::UnisonStream) -> Self {
        Self {
            config,
            stream: Box::new(stream),
            stats: ServiceStats::default(),
            start_time: std::time::Instant::now(),
        }
    }
    
    pub fn get_config(&self) -> &ServiceConfig {
        &self.config
    }
    
    pub fn get_stats(&self) -> &ServiceStats {
        &self.stats
    }
    
    pub fn update_stats<F>(&mut self, updater: F) 
    where 
        F: FnOnce(&mut ServiceStats)
    {
        updater(&mut self.stats);
    }
}

impl SystemStream for UnisonService {
    async fn send(&mut self, data: serde_json::Value) -> Result<(), NetworkError> {
        self.stream.send(data).await
    }
    
    async fn receive(&mut self) -> Result<serde_json::Value, NetworkError> {
        self.stream.receive().await
    }
    
    fn is_active(&self) -> bool {
        self.stream.is_active()
    }
    
    async fn close(&mut self) -> Result<(), NetworkError> {
        self.stream.close().await
    }
    
    fn get_handle(&self) -> StreamHandle {
        self.stream.get_handle()
    }
}

impl Service for UnisonService {
    fn service_type(&self) -> &str {
        "unison-service"
    }
    
    fn service_name(&self) -> &str {
        &self.config.service_name
    }
    
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::from([
            ("service_name".to_string(), self.config.service_name.clone()),
            ("service_version".to_string(), self.config.service_version.clone()),
            ("uptime_seconds".to_string(), self.start_time.elapsed().as_secs().to_string()),
            ("requests_processed".to_string(), self.stats.requests_processed.to_string()),
        ])
    }
    
    fn version(&self) -> &str {
        &self.config.service_version
    }
    
    async fn handle_request(
        &mut self, 
        method: &str, 
        _payload: serde_json::Value
    ) -> Result<serde_json::Value, NetworkError> {
        self.stats.requests_processed += 1;
        
        match method {
            "ping" => {
                Ok(serde_json::json!({
                    "service": self.service_name(),
                    "version": self.version(),
                    "status": "healthy",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            },
            "get_stats" => {
                self.stats.uptime_seconds = self.start_time.elapsed().as_secs();
                Ok(serde_json::to_value(&self.stats).unwrap())
            },
            "get_capabilities" => {
                Ok(serde_json::json!({
                    "capabilities": self.get_capabilities(),
                    "service": self.service_name()
                }))
            },
            _ => {
                self.stats.errors_count += 1;
                Err(NetworkError::HandlerNotFound { method: method.to_string() })
            }
        }
    }
    
    fn get_capabilities(&self) -> Vec<String> {
        vec![
            "ping".to_string(), 
            "heartbeat".to_string(),
            "get_stats".to_string(),
            "get_capabilities".to_string()
        ]
    }
}

impl RealtimeService for UnisonService {
    async fn send_realtime(
        &mut self, 
        method: &str,
        data: serde_json::Value,
        priority: ServicePriority
    ) -> Result<(), NetworkError> {
        let realtime_data = serde_json::json!({
            "method": method,
            "data": data,
            "priority": priority as u8,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        let metadata = HashMap::from([
            ("message_type".to_string(), "realtime".to_string()),
            ("priority".to_string(), (priority as u8).to_string()),
            ("method".to_string(), method.to_string()),
        ]);
        
        self.send_with_metadata(realtime_data, metadata).await
    }
    
    async fn receive_with_timeout(
        &mut self, 
        timeout: std::time::Duration
    ) -> Result<serde_json::Value, NetworkError> {
        tokio::time::timeout(timeout, self.receive()).await
            .map_err(|_| NetworkError::Timeout)?
    }
    
    async fn get_performance_stats(&self) -> Result<ServiceStats, NetworkError> {
        let mut stats = self.stats.clone();
        stats.uptime_seconds = self.start_time.elapsed().as_secs();
        Ok(stats)
    }
}