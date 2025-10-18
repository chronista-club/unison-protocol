//! パケットヘッダーの定義
//!
//! UnisonPacketのヘッダー構造を定義します。
//! rkyvによるゼロコピーシリアライゼーションをサポートします。

use super::flags::PacketFlags;
use rkyv::{Archive, Deserialize, Serialize};

/// パケットタイプを定義する列挙型
#[derive(Archive, Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[archive(check_bytes)]
#[repr(u8)]
pub enum PacketType {
    /// 通常のデータパケット
    Data = 0x00,
    /// 制御メッセージ
    Control = 0x01,
    /// キープアライブ
    Heartbeat = 0x02,
    /// ハンドシェイク
    Handshake = 0x03,
    /// カスタムタイプ（アプリケーション定義）
    Custom(u8),
}

impl From<u8> for PacketType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Data,
            0x01 => Self::Control,
            0x02 => Self::Heartbeat,
            0x03 => Self::Handshake,
            v => Self::Custom(v),
        }
    }
}

impl From<PacketType> for u8 {
    fn from(pt: PacketType) -> Self {
        match pt {
            PacketType::Data => 0x00,
            PacketType::Control => 0x01,
            PacketType::Heartbeat => 0x02,
            PacketType::Handshake => 0x03,
            PacketType::Custom(v) => v,
        }
    }
}

/// UnisonPacketのヘッダー構造
///
/// 固定長48バイトのヘッダーで、パケットのメタデータを格納します。
#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[archive(check_bytes)]
pub struct UnisonPacketHeader {
    /// プロトコルバージョン（現在: 0x01）
    pub version: u8,

    /// パケットタイプ
    pub packet_type: u8,

    /// ビットフラグ（PacketFlags）
    pub flags: u16,

    /// 圧縮前のペイロード長（バイト）
    pub payload_length: u32,

    /// 圧縮後のペイロード長（0=非圧縮）
    pub compressed_length: u32,

    /// CRC32チェックサム（0=チェックサムなし）
    pub checksum: u32,

    /// シーケンス番号
    pub sequence_number: u64,

    /// タイムスタンプ（Unix timestamp、ナノ秒）
    pub timestamp: u64,

    /// ストリーム識別子
    pub stream_id: u64,

    /// アライメント用パディング
    #[doc(hidden)]
    pub _padding: [u8; 8],
}

impl UnisonPacketHeader {
    /// 現在のプロトコルバージョン
    pub const CURRENT_VERSION: u8 = 0x01;

    /// 新しいヘッダーを作成
    pub fn new(packet_type: PacketType) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            packet_type: packet_type.into(),
            flags: 0,
            payload_length: 0,
            compressed_length: 0,
            checksum: 0,
            sequence_number: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            stream_id: 0,
            _padding: [0; 8],
        }
    }

    /// パケットタイプを取得
    pub fn packet_type(&self) -> PacketType {
        PacketType::from(self.packet_type)
    }

    /// パケットタイプを設定
    pub fn set_packet_type(&mut self, packet_type: PacketType) {
        self.packet_type = packet_type.into();
    }

    /// フラグを取得
    pub fn flags(&self) -> PacketFlags {
        PacketFlags::from(self.flags)
    }

    /// フラグを設定
    pub fn set_flags(&mut self, flags: PacketFlags) {
        self.flags = flags.into();
    }

    /// 圧縮されているかチェック
    pub fn is_compressed(&self) -> bool {
        self.compressed_length > 0 && self.flags().is_compressed()
    }

    /// チェックサムが有効かチェック
    pub fn has_checksum(&self) -> bool {
        self.checksum != 0
    }

    /// バージョンの互換性をチェック
    pub fn is_compatible(&self) -> bool {
        self.version == Self::CURRENT_VERSION
    }

    /// ペイロードの実際のサイズを取得（圧縮されている場合は圧縮後のサイズ）
    pub fn actual_payload_size(&self) -> u32 {
        if self.compressed_length > 0 {
            self.compressed_length
        } else {
            self.payload_length
        }
    }

    /// シーケンス番号を設定
    pub fn with_sequence(mut self, seq: u64) -> Self {
        self.sequence_number = seq;
        self
    }

    /// ストリームIDを設定
    pub fn with_stream_id(mut self, id: u64) -> Self {
        self.stream_id = id;
        self
    }

    /// チェックサムを設定
    pub fn with_checksum(mut self, checksum: u32) -> Self {
        self.checksum = checksum;
        self
    }

    /// 現在のタイムスタンプを更新
    pub fn update_timestamp(&mut self) {
        self.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
    }
}

impl Default for UnisonPacketHeader {
    fn default() -> Self {
        Self::new(PacketType::Data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_creation() {
        let header = UnisonPacketHeader::new(PacketType::Data);
        assert_eq!(header.version, UnisonPacketHeader::CURRENT_VERSION);
        assert_eq!(header.packet_type(), PacketType::Data);
        assert_eq!(header.payload_length, 0);
        assert_eq!(header.compressed_length, 0);
        assert!(!header.is_compressed());
        assert!(!header.has_checksum());
    }

    #[test]
    fn test_packet_type_conversion() {
        assert_eq!(u8::from(PacketType::Data), 0x00);
        assert_eq!(u8::from(PacketType::Control), 0x01);
        assert_eq!(u8::from(PacketType::Heartbeat), 0x02);
        assert_eq!(u8::from(PacketType::Handshake), 0x03);
        assert_eq!(u8::from(PacketType::Custom(0xFF)), 0xFF);

        assert_eq!(PacketType::from(0x00), PacketType::Data);
        assert_eq!(PacketType::from(0x01), PacketType::Control);
        assert_eq!(PacketType::from(0x02), PacketType::Heartbeat);
        assert_eq!(PacketType::from(0x03), PacketType::Handshake);
        assert_eq!(PacketType::from(0xFF), PacketType::Custom(0xFF));
    }

    #[test]
    fn test_flags_integration() {
        let mut header = UnisonPacketHeader::new(PacketType::Data);
        let mut flags = PacketFlags::new();
        flags.set(PacketFlags::COMPRESSED | PacketFlags::PRIORITY_HIGH);

        header.set_flags(flags);
        assert_eq!(header.flags().bits(), flags.bits());
        assert!(header.flags().is_compressed());
        assert!(header.flags().is_high_priority());
    }

    #[test]
    fn test_builder_pattern() {
        let header = UnisonPacketHeader::new(PacketType::Control)
            .with_sequence(42)
            .with_stream_id(1337)
            .with_checksum(0xDEADBEEF);

        assert_eq!(header.sequence_number, 42);
        assert_eq!(header.stream_id, 1337);
        assert_eq!(header.checksum, 0xDEADBEEF);
        assert!(header.has_checksum());
    }

    #[test]
    fn test_actual_payload_size() {
        let mut header = UnisonPacketHeader::new(PacketType::Data);
        header.payload_length = 1000;
        assert_eq!(header.actual_payload_size(), 1000);

        header.compressed_length = 500;
        let mut flags = PacketFlags::new();
        flags.set(PacketFlags::COMPRESSED);
        header.set_flags(flags);
        assert_eq!(header.actual_payload_size(), 500);
    }

    #[test]
    fn test_header_size() {
        // ヘッダーサイズが48バイトであることを確認
        use std::mem::size_of;
        let header_size = size_of::<UnisonPacketHeader>();
        assert_eq!(header_size, 48, "Header size should be exactly 48 bytes");
    }
}
