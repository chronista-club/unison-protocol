# Scripts

このディレクトリには、Unison Protocolの開発・運用に必要なスクリプトが含まれています。

## 証明書生成

### `generate_quic_certs.sh`

QUIC通信用の証明書を生成するスクリプトです。

**使用方法:**
```bash
# 証明書を生成
./scripts/generate_quic_certs.sh
```

**生成される証明書の特徴:**
- **アルゴリズム**: ECDSA P-256カーブ（QUIC/TLS 1.3に最適化）
- **有効期限**: 1000日
- **対象ドメイン**: 
  - `dev.chronista.club`
  - `*.unison.svc.cluster.local` (Kubernetes対応)
  - `localhost`
- **対象IP**: `127.0.0.1`, `0.0.0.0`

**生成されるファイル:**
```
assets/certs/
├── cert.pem        # 証明書 (PEM形式)
├── cert.der        # 証明書 (DER形式) - Rust最適化
├── private_key.pem # 秘密鍵 (PEM形式)
├── private_key.der # 秘密鍵 (DER形式) - Rust最適化
└── README.md       # 証明書についての詳細説明
```

**セキュリティ:**
- 秘密鍵ファイルは自動的に600権限に設定
- 証明書ファイルは644権限に設定
- 開発・テスト専用（本番環境では信頼できるCAを使用）

## 開発時の使用例

```bash
# 1. 証明書生成
./scripts/generate_quic_certs.sh

# 2. サーバー起動
cargo run --example unison_ping_server

# 3. クライアント接続テスト
cargo run --example unison_ping_client
```

## 本番環境での証明書

本番環境では、信頼できるCA（Certificate Authority）から発行された証明書を使用してください：

```bash
# Let's Encryptの例
certbot certonly --standalone -d your-domain.com

# 証明書ファイルをDER形式に変換
openssl x509 -in /etc/letsencrypt/live/your-domain.com/cert.pem \
    -out /path/to/cert.der -outform der

openssl rsa -in /etc/letsencrypt/live/your-domain.com/privkey.pem \
    -out /path/to/private_key.der -outform der
```

## トラブルシューティング

### opensslコマンドが見つからない場合

**macOS:**
```bash
brew install openssl
```

**Ubuntu/Debian:**
```bash
sudo apt-get install openssl
```

**CentOS/RHEL:**
```bash
sudo yum install openssl
```

### 権限エラーが発生する場合

```bash
chmod +x scripts/generate_quic_certs.sh
sudo mkdir -p assets/certs
sudo chown $USER:$USER assets/certs
```