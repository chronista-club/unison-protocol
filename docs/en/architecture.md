# Unison Protocol アーキテクチャ

## システム全体構成

```mermaid
graph TB
    subgraph "Client Layer"
        PC[ProtocolClient]
        UC[UnisonClient]
        TW[TransportWrapper]
        DT[DummyTransport]
        QC[QuicClient]
    end

    subgraph "Server Layer"
        PS[ProtocolServer]
        US[UnisonServer]
        QS[QuicServer]
        PST[ProtocolServerTrait]
    end

    subgraph "Service Layer"
        SW[ServiceWrapper]
        USV[UnisonService]
        SVC[Service Trait]
        SC[ServiceConfig]
    end

    subgraph "Stream Layer"
        SSW[SystemStreamWrapper]
        MSS[MockSystemStream]
        QUS[QuicUnisonStream]
        SS[SystemStream Trait]
        SH[StreamHandle]
    end

    subgraph "Network Layer"
        QE[Quinn Endpoint]
        QConn[Quinn Connection]
        TLS[TLS/Rustls]
        UDP[UDP Socket]
    end

    subgraph "Protocol Layer"
        PM[ProtocolMessage]
        MT[MessageType]
        JSON[JSON Payload]
    end

    subgraph "Parser Layer"
        SP[SchemaParser]
        KDL[KDL Document]
        TR[TypeRegistry]
    end

    %% Client connections
    PC --> TW
    TW --> DT
    TW --> QC
    PC --> SW
    UC --> PC

    %% Server connections
    PS --> QS
    US --> PS
    PS --> SW
    QS --> QE

    %% Service connections
    SW --> USV
    USV --> SSW
    USV --> SC
    SW --> SVC

    %% Stream connections
    SSW --> MSS
    SSW --> QUS
    SSW --> SS
    SS --> SH

    %% Network connections
    QC --> QE
    QS --> QE
    QE --> QConn
    QConn --> TLS
    TLS --> UDP

    %% Protocol connections
    PC --> PM
    PS --> PM
    PM --> MT
    PM --> JSON

    %% Parser connections
    SP --> KDL
    SP --> TR
    KDL --> JSON

    classDef client fill:#e1f5fe
    classDef server fill:#f3e5f5
    classDef service fill:#e8f5e8
    classDef stream fill:#fff3e0
    classDef network fill:#ffebee
    classDef protocol fill:#f1f8e9
    classDef parser fill:#fce4ec

    class PC,UC,TW,DT,QC client
    class PS,US,QS,PST server
    class SW,USV,SVC,SC service
    class SSW,MSS,QUS,SS,SH stream
    class QE,QConn,TLS,UDP network
    class PM,MT,JSON protocol
    class SP,KDL,TR parser
```

## データフロー

```mermaid
sequenceDiagram
    participant Client as ProtocolClient
    participant Transport as TransportWrapper
    participant QUIC as QuicClient
    participant Server as QuicServer
    participant Handler as ProtocolServer
    participant Service as ServiceWrapper

    Client->>Transport: send(message)
    Transport->>QUIC: send(message)
    QUIC->>Server: QUIC connection
    Server->>Handler: handle_call/handle_stream
    Handler->>Service: process request
    Service-->>Handler: response
    Handler-->>Server: result
    Server-->>QUIC: QUIC response
    QUIC-->>Transport: response
    Transport-->>Client: result
```

## Enum Wrapper パターン

```mermaid
classDiagram
    class TransportWrapper {
        <<enumeration>>
        +Dummy(DummyTransport)
        +Quic(QuicClient)
        +send(message)
        +receive()
        +connect(url)
        +disconnect()
    }

    class ServiceWrapper {
        <<enumeration>>
        +Unison(UnisonService)
        +service_name()
        +handle_request(method, payload)
        +send_with_metadata(data, metadata)
        +start_service_heartbeat(interval)
    }

    class SystemStreamWrapper {
        <<enumeration>>
        +Quic(UnisonStream)
        +Mock(MockSystemStream)
        +send(data)
        +receive()
        +close()
        +get_handle()
    }

    TransportWrapper --|> Transport : implements
    ServiceWrapper --|> Service : implements
    ServiceWrapper --|> SystemStream : implements
    SystemStreamWrapper --|> SystemStream : implements
```

## QUIC通信アーキテクチャ

```mermaid
graph LR
    subgraph "Client Side"
        C1[Client App]
        C2[ProtocolClient]
        C3[TransportWrapper::Quic]
        C4[QuicClient]
        C5[Quinn Endpoint]
    end

    subgraph "Network"
        N1[QUIC Connection]
        N2[TLS 1.3]
        N3[UDP Transport]
    end

    subgraph "Server Side"
        S1[QuicServer]
        S2[ProtocolServer]
        S3[ServiceWrapper]
        S4[UnisonService]
        S5[Server App]
    end

    C1 --> C2
    C2 --> C3
    C3 --> C4
    C4 --> C5
    C5 --> N1
    N1 --> N2
    N2 --> N3
    N3 --> N2
    N2 --> N1
    N1 --> S1
    S1 --> S2
    S2 --> S3
    S3 --> S4
    S4 --> S5

    classDef client fill:#e3f2fd
    classDef network fill:#fff3e0
    classDef server fill:#e8f5e8

    class C1,C2,C3,C4,C5 client
    class N1,N2,N3 network
    class S1,S2,S3,S4,S5 server
```

## 非同期処理とdyn互換性

```mermaid
graph TB
    subgraph "Problem: async trait + Box<dyn Trait>"
        AT[async trait methods]
        BDT[Box<dyn Trait>]
        DYN[dyn compatibility]
        AT -.->|incompatible| BDT
        BDT -.->|requires| DYN
        AT -.->|breaks| DYN
    end

    subgraph "Solution: Enum Wrapper Pattern"
        EW[Enum Wrapper]
        CT[Concrete Types]
        AT2[async fn support]
        EW --> CT
        EW --> AT2
        CT --> AT2
    end

    subgraph "Benefits"
        TC[Type Safety]
        PERF[Performance]
        COMP[Compatibility]
        MAINT[Maintainability]
    end

    EW --> TC
    EW --> PERF
    EW --> COMP
    EW --> MAINT

    classDef problem fill:#ffebee
    classDef solution fill:#e8f5e8
    classDef benefit fill:#e3f2fd

    class AT,BDT,DYN problem
    class EW,CT,AT2 solution
    class TC,PERF,COMP,MAINT benefit
```

## モジュール依存関係

```mermaid
graph TD
    subgraph "src/"
        L[lib.rs]
        
        subgraph "network/"
            NM[mod.rs]
            NC[client.rs]
            NS[server.rs]
            NQ[quic.rs]
            NSV[service.rs]
        end
        
        subgraph "parser/"
            PM[mod.rs]
            PS[schema.rs]
        end
        
        subgraph "codegen/"
            CM[mod.rs]
            CR[rust.rs]
            CT[typescript.rs]
        end
    end

    L --> NM
    L --> PM
    L --> CM
    
    NM --> NC
    NM --> NS
    NM --> NQ
    NM --> NSV
    
    NC --> NQ
    NS --> NQ
    NS --> NSV
    NSV --> NM
    
    PM --> PS
    CM --> CR
    CM --> CT

    classDef main fill:#e3f2fd
    classDef network fill:#e8f5e8
    classDef parser fill:#fff3e0
    classDef codegen fill:#f3e5f5

    class L main
    class NM,NC,NS,NQ,NSV network
    class PM,PS parser
    class CM,CR,CT codegen
```

## 主要コンポーネント詳細

### ProtocolClient
- **役割**: クライアント側エントリーポイント
- **機能**: 
  - Transport管理 (QUIC/Dummy)
  - Service登録・管理
  - RPC呼び出し (call/stream)
- **依存**: TransportWrapper, ServiceWrapper

### ProtocolServer  
- **役割**: サーバー側エントリーポイント
- **機能**:
  - ハンドラー登録 (call/stream/unison)
  - Service管理
  - QUIC接続処理
- **依存**: QuicServer, ServiceWrapper

### QuicServer
- **役割**: QUIC通信サーバー
- **機能**:
  - Quinn Endpoint管理
  - TLS証明書自動ロード
  - 接続受付・処理
- **依存**: Quinn, Rustls

### ServiceWrapper
- **役割**: Service trait dyn互換性解決
- **機能**:
  - Service trait完全実装  
  - SystemStream trait実装
  - UnisonService ラッピング
- **依存**: UnisonService

### SystemStreamWrapper
- **役割**: SystemStream trait dyn互換性解決
- **機能**:
  - 双方向ストリーム抽象化
  - Mock/QUIC実装切り替え
  - ストリームハンドル管理
- **依存**: MockSystemStream, QuicUnisonStream

### TransportWrapper  
- **役割**: Transport trait dyn互換性解決
- **機能**:
  - Dummy/QUIC転送実装
  - 接続管理
  - メッセージ送受信
- **依存**: DummyTransport, QuicClient

## 設計原則

1. **型安全性**: Enum wrapperによる静的型チェック
2. **パフォーマンス**: Box<dyn>オーバーヘッドなし
3. **拡張性**: 新しい実装の追加容易
4. **互換性**: async fn ネイティブサポート
5. **保守性**: 明確な責任分離とモジュール構造