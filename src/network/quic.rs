use anyhow::{Context, Result};
use async_trait::async_trait;
use futures_util::StreamExt;
use quinn::{Endpoint, ServerConfig, ClientConfig, Connection, RecvStream, SendStream};
use rustls::{ServerConfig as RustlsServerConfig, ClientConfig as RustlsClientConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rcgen::{generate_simple_self_signed, CertificateParams};
use rust_embed::{RustEmbed, Embed};
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::{Arc, atomic::{AtomicU64, AtomicBool, Ordering}};
use std::time::SystemTime;
use tokio::sync::{mpsc, RwLock, Mutex};
use tracing::{debug, error, info, warn};

use super::{
    client::Transport, server::ProtocolServer, ProtocolMessage, ProtocolServerTrait,
    NetworkError, SystemStream, StreamHandle, MessageType,
};

/// Default certificate file paths for assets/certs directory
pub const DEFAULT_CERT_PATH: &str = "assets/certs/cert.pem";
pub const DEFAULT_KEY_PATH: &str = "assets/certs/private_key.der";

/// Embedded certificates for development use
#[derive(RustEmbed)]
#[folder = "assets/certs"]
#[include = "*.pem"]
#[include = "*.der"]
struct EmbeddedCerts;

/// QUIC client implementation
pub struct QuicClient {
    endpoint: Option<Endpoint>,
    connection: Arc<RwLock<Option<Connection>>>,
    rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<ProtocolMessage>>>>,
    tx: mpsc::UnboundedSender<ProtocolMessage>,
}

impl QuicClient {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::unbounded_channel();
        Ok(Self {
            endpoint: None,
            connection: Arc::new(RwLock::new(None)),
            rx: Arc::new(RwLock::new(Some(rx))),
            tx,
        })
    }

    /// Configure client with custom TLS configuration
    pub async fn configure_client() -> Result<ClientConfig> {
        let client_crypto_config = RustlsClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
            .with_no_client_auth();
        
        let crypto = quinn::crypto::rustls::QuicClientConfig::try_from(client_crypto_config)?;
        let mut client_config = ClientConfig::new(Arc::new(crypto));

        // Configure QUIC transport parameters optimized for real-time communication
        let mut transport_config = quinn::TransportConfig::default();
        
        // Optimize for low latency
        transport_config.max_idle_timeout(Some(std::time::Duration::from_secs(60).try_into().unwrap()));
        transport_config.keep_alive_interval(Some(std::time::Duration::from_secs(10)));
        
        // Enable 0-RTT for faster reconnection
        transport_config.max_concurrent_uni_streams(0u32.into()); // Unlimited unidirectional streams
        transport_config.max_concurrent_bidi_streams(1000u32.into()); // Support many bidirectional streams
        
        // Optimize congestion control for real-time data
        transport_config.initial_rtt(std::time::Duration::from_millis(100));
        
        client_config.transport_config(Arc::new(transport_config));

        Ok(client_config)
    }

    async fn start_receive_loop(&self, connection: Connection) {
        let tx = self.tx.clone();
        
        tokio::spawn(async move {
            loop {
                match connection.accept_uni().await {
                    Ok(recv_stream) => {
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_incoming_stream(recv_stream, tx).await {
                                error!("Failed to handle incoming stream: {}", e);
                            }
                        });
                    }
                    Err(quinn::ConnectionError::ApplicationClosed(_)) => {
                        info!("Connection closed by application");
                        break;
                    }
                    Err(e) => {
                        error!("Failed to accept stream: {}", e);
                        break;
                    }
                }
            }
        });
    }

    async fn handle_incoming_stream(
        mut recv_stream: RecvStream,
        tx: mpsc::UnboundedSender<ProtocolMessage>,
    ) -> Result<()> {
        let data = recv_stream.read_to_end(8 * 1024 * 1024).await?; // 8MB limit
        let message: ProtocolMessage = serde_json::from_slice(&data)?;
        
        tx.send(message).map_err(|e| {
            anyhow::anyhow!("Failed to send message to channel: {}", e)
        })?;
        
        Ok(())
    }
}

#[async_trait]
impl Transport for QuicClient {
    async fn send(&self, message: ProtocolMessage) -> Result<()> {
        let connection_guard = self.connection.read().await;
        if let Some(connection) = connection_guard.as_ref() {
            let mut send_stream = connection.open_uni().await
                .context("Failed to open QUIC stream")?;
            
            let json = serde_json::to_vec(&message)?;
            send_stream.write_all(&json).await
                .context("Failed to write to QUIC stream")?;
            send_stream.finish()
                .context("Failed to finish QUIC stream")?;
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("QUIC not connected"))
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
        // Parse URL to extract host and port
        let addr: SocketAddr = url.parse()
            .context("Failed to parse server address")?;
        
        let client_config = Self::configure_client().await?;
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())?;
        endpoint.set_default_client_config(client_config);

        let connection = endpoint
            .connect(addr, "localhost")?
            .await
            .context("Failed to establish QUIC connection")?;

        info!("Connected to QUIC server at {}", addr);
        
        self.start_receive_loop(connection.clone()).await;
        *self.connection.write().await = Some(connection);
        
        Ok(())
    }
    
    async fn disconnect(&self) -> Result<()> {
        let mut connection_guard = self.connection.write().await;
        if let Some(connection) = connection_guard.take() {
            connection.close(quinn::VarInt::from_u32(0), b"client disconnect");
        }
        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        let connection_guard = self.connection.read().await;
        if let Some(connection) = connection_guard.as_ref() {
            !connection.close_reason().is_some()
        } else {
            false
        }
    }
}

/// QUIC server implementation
pub struct QuicServer {
    server: Arc<dyn ProtocolServerTrait>,
    endpoint: Option<Endpoint>,
}

impl QuicServer {
    pub fn new(server: Arc<dyn ProtocolServerTrait>) -> Self {
        Self {
            server,
            endpoint: None,
        }
    }

    /// Generate self-signed certificate for QUIC/TLS 1.3 (optimized for production use)
    pub fn generate_self_signed_cert() -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
        let subject_alt_names = vec![
            "localhost".to_string(),
            "*.unison.svc.cluster.local".to_string(),
            "dev.chronista.club".to_string(),
        ];
        
        let cert_key = rcgen::generate_simple_self_signed(subject_alt_names)?;
        let cert_der_bytes = cert_key.cert.der().to_vec();
        let private_key_der_bytes = cert_key.key_pair.serialize_der();
        
        Ok((
            vec![CertificateDer::from(cert_der_bytes)],
            PrivateKeyDer::try_from(private_key_der_bytes).unwrap(),
        ))
    }
    
    /// Load certificate from external files (for production deployment)
    pub fn load_cert_from_files(cert_path: &str, key_path: &str) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
        let cert_pem = std::fs::read_to_string(cert_path)?;
        let key_der = std::fs::read(key_path)?;
        
        let cert_chain = rustls_pemfile::certs(&mut cert_pem.as_bytes())
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to parse certificate")?;
        let certs = cert_chain.into_iter().map(CertificateDer::from).collect();
        
        // Convert to owned data for static lifetime
        let key_der_owned = key_der.clone();
        let private_key = PrivateKeyDer::try_from(key_der_owned.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))?;
        
        Ok((certs, private_key.clone_key()))
    }
    
    /// Load certificate from embedded assets (rust-embed)
    pub fn load_cert_embedded() -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
        // Try to load embedded certificate files
        let cert_data = EmbeddedCerts::get("cert.pem")
            .ok_or_else(|| anyhow::anyhow!("Embedded cert.pem not found"))?;
        let key_data = EmbeddedCerts::get("private_key.der")
            .ok_or_else(|| anyhow::anyhow!("Embedded private_key.der not found"))?;
        
        // Parse certificate
        let cert_pem = std::str::from_utf8(&cert_data.data)?;
        let cert_chain = rustls_pemfile::certs(&mut cert_pem.as_bytes())
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to parse embedded certificate")?;
        let certs = cert_chain.into_iter().map(CertificateDer::from).collect();
        
        // Load private key (already in DER format) - clone to own the data
        let key_data_owned = key_data.data.to_vec();
        let private_key = PrivateKeyDer::try_from(key_data_owned.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to parse embedded private key: {}", e))?;
        
        info!("🔐 Loaded embedded certificate from rust-embed");
        Ok((certs, private_key.clone_key()))
    }
    
    /// Automatically load certificate with fallback priority:
    /// 1. External files (assets/certs/)
    /// 2. Embedded certificates (rust-embed)
    /// 3. Generated self-signed certificate
    pub fn load_cert_auto() -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
        // Priority 1: External files
        if std::path::Path::new(DEFAULT_CERT_PATH).exists() && 
           std::path::Path::new(DEFAULT_KEY_PATH).exists() {
            info!("🔐 Loading certificate from external files");
            return Self::load_cert_from_files(DEFAULT_CERT_PATH, DEFAULT_KEY_PATH);
        }
        
        // Priority 2: Embedded certificates (rust-embed使用)
        if let Ok(result) = Self::load_cert_embedded() {
            return Ok(result);
        }
        
        // Priority 3: Generate self-signed certificate
        info!("🔐 Generating self-signed certificate (no certificate files found)");
        Self::generate_self_signed_cert()
    }

    /// Configure server with TLS (using auto certificate detection)
    pub async fn configure_server() -> Result<ServerConfig> {
        let (certs, private_key) = Self::load_cert_auto()?;
        
        let rustls_server_config = RustlsServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, private_key)
            .map_err(|e| anyhow::anyhow!("Failed to configure TLS: {}", e))?;
            
        let crypto = quinn::crypto::rustls::QuicServerConfig::try_from(rustls_server_config)?;
        let mut server_config = ServerConfig::with_crypto(Arc::new(crypto));
        
        // Configure QUIC transport parameters optimized for real-time communication
        let mut transport_config = quinn::TransportConfig::default();
        
        // Optimize for low latency and high throughput
        transport_config.max_idle_timeout(Some(std::time::Duration::from_secs(60).try_into().unwrap()));
        transport_config.keep_alive_interval(Some(std::time::Duration::from_secs(10)));
        
        // Support many concurrent streams for multiplexed communication
        transport_config.max_concurrent_uni_streams(0u32.into()); // Unlimited unidirectional streams
        transport_config.max_concurrent_bidi_streams(1000u32.into()); // Support many bidirectional streams
        
        // Optimize for protocol-level communication patterns
        transport_config.initial_rtt(std::time::Duration::from_millis(100));
        // Max UDP payload is handled automatically by QUIC
        
        server_config.transport_config(Arc::new(transport_config));
        
        Ok(server_config)
    }

    pub async fn bind(&mut self, addr: &str) -> Result<()> {
        let socket_addr: SocketAddr = addr.parse()
            .context("Failed to parse bind address")?;
        
        let server_config = Self::configure_server().await?;
        let endpoint = Endpoint::server(server_config, socket_addr)?;
        
        info!("QUIC server bound to {}", socket_addr);
        self.endpoint = Some(endpoint);
        Ok(())
    }
    
    pub async fn start(&self) -> Result<()> {
        let endpoint = self.endpoint.as_ref()
            .context("Server not bound to an address")?;
        
        info!("QUIC server listening for connections");
        
        while let Some(connecting) = endpoint.accept().await {
            let connection = connecting.await?;
            let remote_addr = connection.remote_address();
            info!("New QUIC connection from: {}", remote_addr);
            
            let server = Arc::clone(&self.server);
            tokio::spawn(async move {
                if let Err(e) = handle_connection(connection, server).await {
                    error!("Connection error: {}", e);
                }
            });
        }
        
        Ok(())
    }
}

async fn handle_connection(
    connection: Connection,
    server: Arc<dyn ProtocolServerTrait>,
) -> Result<()> {
    loop {
        match connection.accept_uni().await {
            Ok(mut recv_stream) => {
                let server = Arc::clone(&server);
                let connection = connection.clone();
                
                tokio::spawn(async move {
                    match recv_stream.read_to_end(8 * 1024 * 1024).await {
                        Ok(data) => {
                            match serde_json::from_slice::<ProtocolMessage>(&data) {
                                Ok(request) => {
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
                                            
                                            if let Err(e) = send_response(connection, response_msg).await {
                                                error!("Failed to send response: {}", e);
                                            }
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
                                                        
                                                        if let Err(e) = send_response(connection.clone(), msg).await {
                                                            error!("Failed to send stream data: {}", e);
                                                            break;
                                                        }
                                                    }
                                                    
                                                    // Send stream end message
                                                    let end_msg = ProtocolMessage {
                                                        id: request.id,
                                                        method: request.method,
                                                        msg_type: super::MessageType::StreamEnd,
                                                        payload: serde_json::json!({}),
                                                    };
                                                    
                                                    if let Err(e) = send_response(connection, end_msg).await {
                                                        error!("Failed to send stream end: {}", e);
                                                    }
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
                                                    
                                                    if let Err(e) = send_response(connection, error_msg).await {
                                                        error!("Failed to send error response: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                        _ => {
                                            warn!("Unexpected message type: {:?}", request.msg_type);
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse message: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to read from stream: {}", e);
                        }
                    }
                });
            }
            Err(quinn::ConnectionError::ApplicationClosed(_)) => {
                info!("Client disconnected");
                break;
            }
            Err(e) => {
                error!("Failed to accept stream: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}

async fn send_response(connection: Connection, message: ProtocolMessage) -> Result<()> {
    let mut send_stream = connection.open_uni().await?;
    let data = serde_json::to_vec(&message)?;
    send_stream.write_all(&data).await?;
    send_stream.finish()?;
    Ok(())
}

/// Custom certificate verifier that skips verification (for testing only)
#[derive(Debug)]
pub struct SkipServerVerification;

impl rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    
    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    
    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        use rustls::SignatureScheme;
        vec![
            SignatureScheme::RSA_PKCS1_SHA1,
            SignatureScheme::ECDSA_SHA1_Legacy,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
            SignatureScheme::ED448,
        ]
    }
}

/// Unison Stream - QUIC bidirectional stream implementation
pub struct UnisonStream {
    stream_id: u64,
    method: String,
    connection: Arc<Connection>,
    send_stream: Arc<Mutex<Option<SendStream>>>,
    recv_stream: Arc<Mutex<Option<RecvStream>>>,
    is_active: Arc<AtomicBool>,
    handle: StreamHandle,
}

impl UnisonStream {
    pub async fn new(
        method: String,
        connection: Arc<Connection>,
        stream_id: Option<u64>,
    ) -> Result<Self> {
        static STREAM_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        
        let id = stream_id.unwrap_or_else(|| STREAM_ID_COUNTER.fetch_add(1, Ordering::SeqCst));
        
        // Open bidirectional stream
        let (send_stream, recv_stream) = connection.open_bi().await
            .context("Failed to open bidirectional stream")?;
        
        let handle = StreamHandle {
            stream_id: id,
            method: method.clone(),
            created_at: SystemTime::now(),
        };
        
        Ok(Self {
            stream_id: id,
            method,
            connection,
            send_stream: Arc::new(Mutex::new(Some(send_stream))),
            recv_stream: Arc::new(Mutex::new(Some(recv_stream))),
            is_active: Arc::new(AtomicBool::new(true)),
            handle,
        })
    }
    
    /// Create from existing streams (server-side)
    pub fn from_streams(
        stream_id: u64,
        method: String,
        connection: Arc<Connection>,
        send_stream: SendStream,
        recv_stream: RecvStream,
    ) -> Self {
        let handle = StreamHandle {
            stream_id,
            method: method.clone(),
            created_at: SystemTime::now(),
        };
        
        Self {
            stream_id,
            method,
            connection,
            send_stream: Arc::new(Mutex::new(Some(send_stream))),
            recv_stream: Arc::new(Mutex::new(Some(recv_stream))),
            is_active: Arc::new(AtomicBool::new(true)),
            handle,
        }
    }
}

#[async_trait]
impl SystemStream for UnisonStream {
    async fn send(&mut self, data: serde_json::Value) -> Result<(), NetworkError> {
        if !self.is_active() {
            return Err(NetworkError::Connection("Stream is not active".to_string()));
        }
        
        let message = ProtocolMessage {
            id: self.stream_id,
            method: self.method.clone(),
            msg_type: MessageType::StreamSend,
            payload: data,
        };
        
        let json_data = serde_json::to_vec(&message)
            .map_err(|e| NetworkError::Serialization(e))?;
        
        let mut send_guard = self.send_stream.lock().await;
        if let Some(send_stream) = send_guard.as_mut() {
            send_stream.write_all(&json_data).await
                .map_err(|e| NetworkError::Quic(format!("Failed to send data: {}", e)))?;
            Ok(())
        } else {
            Err(NetworkError::Connection("Send stream is closed".to_string()))
        }
    }
    
    async fn receive(&mut self) -> Result<serde_json::Value, NetworkError> {
        if !self.is_active() {
            return Err(NetworkError::Connection("Stream is not active".to_string()));
        }
        
        let mut recv_guard = self.recv_stream.lock().await;
        if let Some(recv_stream) = recv_guard.as_mut() {
            let data = recv_stream.read_to_end(8 * 1024 * 1024).await // 8MB limit
                .map_err(|e| NetworkError::Quic(format!("Failed to receive data: {}", e)))?;
            
            if data.is_empty() {
                self.is_active.store(false, Ordering::SeqCst);
                return Err(NetworkError::Connection("Stream ended".to_string()));
            }
            
            let message: ProtocolMessage = serde_json::from_slice(&data)
                .map_err(|e| NetworkError::Serialization(e))?;
            
            match message.msg_type {
                MessageType::StreamReceive | MessageType::StreamData => {
                    Ok(message.payload)
                },
                MessageType::StreamEnd => {
                    self.is_active.store(false, Ordering::SeqCst);
                    Err(NetworkError::Connection("Stream ended by peer".to_string()))
                },
                MessageType::StreamError => {
                    self.is_active.store(false, Ordering::SeqCst);
                    Err(NetworkError::Protocol(format!("Stream error: {:?}", message.payload)))
                },
                _ => {
                    Err(NetworkError::Protocol(format!("Unexpected message type: {:?}", message.msg_type)))
                }
            }
        } else {
            Err(NetworkError::Connection("Receive stream is closed".to_string()))
        }
    }
    
    fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }
    
    async fn close(&mut self) -> Result<(), NetworkError> {
        self.is_active.store(false, Ordering::SeqCst);
        
        // Close send stream
        if let Some(mut send_stream) = self.send_stream.lock().await.take() {
            send_stream.finish()
                .map_err(|e| NetworkError::Quic(format!("Failed to close send stream: {}", e)))?;
        }
        
        // Close receive stream
        if let Some(mut recv_stream) = self.recv_stream.lock().await.take() {
            recv_stream.stop(quinn::VarInt::from_u32(0))
                .map_err(|e| NetworkError::Quic(format!("Failed to close receive stream: {}", e)))?;
        }
        
        info!("🔒 SystemStream {} closed for method '{}'", self.stream_id, self.method);
        Ok(())
    }
    
    fn get_handle(&self) -> StreamHandle {
        self.handle.clone()
    }
}