# 🔐 Unison Protocol 証明書

このディレクトリには、QUIC通信用のTLS証明書が格納されます。

## 📁 ファイル構成

```
assets/certs/
├── cert.pem        # 証明書 (PEM形式) - 人間が読める形式
├── cert.der        # 証明書 (DER形式) - バイナリ形式、Rust最適化
├── private_key.pem # 秘密鍵 (PEM形式) - 人間が読める形式
├── private_key.der # 秘密鍵 (DER形式) - バイナリ形式、Rust最適化
└── README.md       # このファイル
```

## 🔑 証明書の生成

### 自動生成スクリプト

```bash
# プロジェクトルートから実行
./scripts/generate_quic_certs.sh
```

このスクリプトは以下の処理を行います：

1. **P-256 ECDSAカーブ**を使用した秘密鍵生成（QUIC/TLS 1.3最適化）
2. 自己署名証明書の生成
3. PEM・DER両形式での保存
4. 適切なファイル権限の設定

### 手動生成

```bash
# 1. EC秘密鍵の生成 (P-256カーブ - QUIC最適化)
openssl ecparam -name prime256v1 -genkey -out assets/certs/private_key.pem

# 2. PKCS8 DER形式に変換 (Rust最適化)
openssl pkcs8 -topk8 -in assets/certs/private_key.pem -nocrypt \
    -out assets/certs/private_key.der -outform der

# 3. 自己署名証明書の生成
openssl req -new -x509 \
    -key assets/certs/private_key.der -keyform der \
    -out assets/certs/cert.pem \
    -subj "/CN=dev.chronista.club" \
    -addext "subjectAltName = DNS:*.unison.svc.cluster.local,DNS:localhost,IP:0.0.0.0,IP:127.0.0.1" \
    -days 1000

# 4. DER形式に変換 (Rust最適化)
openssl x509 -in assets/certs/cert.pem -out assets/certs/cert.der -outform der

# 5. 権限設定
chmod 600 assets/certs/private_key.*
chmod 644 assets/certs/cert.*
```

## 🔧 Rustコードでの使用

### 自動検出（推奨）

```rust
use unison_protocol::network::quic::QuicServer;

// 証明書が存在する場合は自動読み込み、なければ自動生成
let (certs, private_key) = QuicServer::load_cert_auto()?;
```

### 明示的な読み込み

```rust
use unison_protocol::network::quic::QuicServer;

// 証明書ファイルから読み込み
let (certs, private_key) = QuicServer::load_cert_from_files(
    "assets/certs/cert.pem",
    "assets/certs/private_key.der"
)?;
```

### 自動生成

```rust
use unison_protocol::network::quic::QuicServer;

// 動的に自己署名証明書を生成
let (certs, private_key) = QuicServer::generate_self_signed_cert()?;
```

## 🌐 対応ドメイン・IP

生成される証明書は以下のドメイン・IPアドレスをカバーします：

### ドメイン名
- `dev.chronista.club` - 開発用メインドメイン
- `*.unison.svc.cluster.local` - Kubernetes クラスタ内通信
- `localhost` - ローカル開発

### IPアドレス
- `127.0.0.1` - ローカルループバック
- `0.0.0.0` - 全インターフェース (開発用)

## 🔒 セキュリティ

### 開発・テスト環境
- **自己署名証明書**: 信頼できるCAに依存しない
- **P-256カーブ**: QUIC/TLS 1.3で最適なパフォーマンス
- **ファイル権限**: 秘密鍵は600、証明書は644

### 本番環境
本番環境では信頼できるCA（Certificate Authority）から発行された証明書を使用してください：

```bash
# Let's Encrypt例
certbot certonly --standalone -d your-domain.com

# 証明書をDER形式に変換
openssl x509 -in /etc/letsencrypt/live/your-domain.com/cert.pem \
    -out assets/certs/cert.der -outform der

openssl rsa -in /etc/letsencrypt/live/your-domain.com/privkey.pem \
    -out assets/certs/private_key.der -outform der
```

## ⚡ QUIC最適化設定

### P-256カーブの利点
- **高速**: RSAより高速な署名・検証
- **省メモリ**: 小さなキーサイズ（256bit）
- **標準準拠**: TLS 1.3推奨カーブ
- **QUIC最適化**: HTTP/3で使用される標準

### TLS 1.3の利点
- **0-RTT**: 高速な接続再開
- **前方秘匿性**: セッションキーの独立性
- **簡素化**: ハンドシェイクの最適化

## 🚨 注意事項

### ⚠️ 開発専用
このディレクトリの証明書は**開発・テスト専用**です：

- 自己署名のため、ブラウザで警告が表示されます
- 本番環境では信頼できるCAの証明書を使用してください
- 秘密鍵をバージョン管理にコミットしないでください

### 🔄 証明書の更新
- 有効期限: **1000日**（約2.7年）
- 期限切れ前に新しい証明書を生成してください
- 自動更新スクリプトの利用を推奨

### 🗑️ 証明書のクリーンアップ

```bash
# 証明書ファイルの削除
rm -f assets/certs/*.pem assets/certs/*.der

# 新規生成
./scripts/generate_quic_certs.sh
```

## 📚 関連リンク

- [QUIC Protocol RFC 9000](https://tools.ietf.org/html/rfc9000)
- [TLS 1.3 RFC 8446](https://tools.ietf.org/html/rfc8446)
- [P-256 Curve (RFC 5480)](https://tools.ietf.org/html/rfc5480)
- [Unison Protocol Documentation](../../docs/)

---

**重要**: このファイル（README.md）はバージョン管理に含まれますが、実際の証明書ファイル（*.pem、*.der）は除外されます。