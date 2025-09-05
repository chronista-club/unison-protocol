#!/bin/bash

# QUIC用証明書生成スクリプト
# P-256 curve (ECDSA) を使用してQUIC/TLS 1.3に最適化された証明書を生成

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CERTS_DIR="$PROJECT_ROOT/assets/certs"

# 証明書ディレクトリの作成
mkdir -p "$CERTS_DIR"

echo "🔐 QUIC用証明書を生成中..."
echo "📁 証明書保存先: $CERTS_DIR"

# 1. EC秘密鍵の生成 (P-256カーブを使用 - QUIC/TLS 1.3に最適)
echo "🔑 EC秘密鍵を生成中 (P-256カーブ)..."
openssl ecparam -name prime256v1 -genkey -out "$CERTS_DIR/private_key.pem"

# 2. 秘密鍵をPKCS8 DER形式に変換 (Rustのquinn crateで直接読み込み可能)
echo "🔄 秘密鍵をPKCS8 DER形式に変換中..."
openssl pkcs8 -topk8 -in "$CERTS_DIR/private_key.pem" -nocrypt -out "$CERTS_DIR/private_key.der" -outform der

# 3. 自己署名証明書の生成 (開発・テスト用)
echo "📜 自己署名証明書を生成中..."
openssl req -new -x509 \
    -key "$CERTS_DIR/private_key.der" -keyform der \
    -out "$CERTS_DIR/cert.pem" \
    -subj "/CN=dev.chronista.club" \
    -addext "subjectAltName = DNS:*.unison.svc.cluster.local,DNS:localhost,IP:0.0.0.0,IP:127.0.0.1" \
    -days 1000

# 4. 証明書をDER形式に変換 (Rustでの読み込みに最適化)
echo "🔄 証明書をDER形式に変換中..."
openssl x509 -in "$CERTS_DIR/cert.pem" -out "$CERTS_DIR/cert.der" -outform der

# 5. 証明書情報の表示
echo ""
echo "✅ 証明書生成完了!"
echo ""
echo "📋 生成された証明書情報:"
echo "=========================="
echo "📁 ファイル:"
echo "  • 秘密鍵 (PEM): $CERTS_DIR/private_key.pem"
echo "  • 秘密鍵 (DER): $CERTS_DIR/private_key.der"
echo "  • 証明書 (PEM): $CERTS_DIR/cert.pem"
echo "  • 証明書 (DER): $CERTS_DIR/cert.der"
echo ""

# 証明書の詳細表示
echo "📜 証明書詳細:"
echo "=============="
openssl x509 -in "$CERTS_DIR/cert.pem" -text -noout | grep -E "(Subject:|DNS:|IP Address:|Not Before:|Not After :)"

echo ""
echo "🔧 Rust実装での使用方法:"
echo "========================"
echo "// 証明書ファイルから読み込む場合:"
echo "let (certs, private_key) = QuicServer::load_cert_from_files("
echo "    \"assets/certs/cert.pem\","
echo "    \"assets/certs/private_key.der\""
echo ")?;"
echo ""
echo "// または、自動生成を使用する場合 (推奨):"
echo "let (certs, private_key) = QuicServer::generate_self_signed_cert()?;"

echo ""
echo "⚠️  注意事項:"
echo "============"
echo "• この証明書は開発・テスト専用です"
echo "• 本番環境では信頼できるCAから発行された証明書を使用してください"
echo "• P-256カーブはQUIC/TLS 1.3で最適なパフォーマンスを提供します"
echo "• 証明書の有効期限は1000日です"

echo ""
echo "🚀 テストコマンド:"
echo "================"
echo "# サーバー起動:"
echo "cargo run --example unison_ping_server"
echo ""
echo "# クライアント接続:"
echo "cargo run --example unison_ping_client"

# 権限設定 (秘密鍵を保護)
chmod 600 "$CERTS_DIR/private_key.pem" "$CERTS_DIR/private_key.der"
chmod 644 "$CERTS_DIR/cert.pem" "$CERTS_DIR/cert.der"

echo ""
echo "🔒 ファイル権限を設定完了 (秘密鍵: 600, 証明書: 644)"
echo ""
echo "✨ 証明書生成が完了しました！"