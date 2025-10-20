//! フレームフラグの定義とビット操作ユーティリティ
//!
//! UnisonPacketで使用されるビットフラグを定義します。
//! 各フラグはパケットの状態や処理方法を示します。

use std::fmt;

/// フレームフラグを表すビットフィールド
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrameFlags(pub u16);

impl FrameFlags {
    /// ペイロードが圧縮されている
    pub const COMPRESSED: u16 = 0b0000_0000_0000_0001; // bit 0

    /// ペイロードが暗号化されている（将来の拡張用）
    pub const ENCRYPTED: u16 = 0b0000_0000_0000_0010; // bit 1

    /// 分割パケットの一部
    pub const FRAGMENTED: u16 = 0b0000_0000_0000_0100; // bit 2

    /// 分割パケットの最後
    pub const LAST_FRAGMENT: u16 = 0b0000_0000_0000_1000; // bit 3

    /// 高優先度パケット
    pub const PRIORITY_HIGH: u16 = 0b0000_0000_0001_0000; // bit 4

    /// ACK要求
    pub const REQUIRES_ACK: u16 = 0b0000_0000_0010_0000; // bit 5

    /// ACKパケット
    pub const IS_ACK: u16 = 0b0000_0000_0100_0000; // bit 6

    /// キープアライブパケット
    pub const KEEPALIVE: u16 = 0b0000_0000_1000_0000; // bit 7

    /// エラー情報を含む
    pub const ERROR: u16 = 0b0000_0001_0000_0000; // bit 8

    /// メタデータ付き
    pub const METADATA: u16 = 0b0000_0010_0000_0000; // bit 9

    /// チェックサム付き
    pub const CHECKSUM: u16 = 0b0000_0100_0000_0000; // bit 10

    // bit 11-15: 将来の拡張用に予約

    /// 新しい空のフラグセットを作成
    pub fn new() -> Self {
        Self(0)
    }

    /// 指定したビットフラグから作成
    pub fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    /// 生のビット値を取得
    pub fn bits(&self) -> u16 {
        self.0
    }

    /// フラグを設定
    pub fn set(&mut self, flag: u16) {
        self.0 |= flag;
    }

    /// フラグをクリア
    pub fn unset(&mut self, flag: u16) {
        self.0 &= !flag;
    }

    /// フラグをトグル
    pub fn toggle(&mut self, flag: u16) {
        self.0 ^= flag;
    }

    /// フラグが設定されているかチェック
    pub fn contains(&self, flag: u16) -> bool {
        self.0 & flag != 0
    }

    /// 複数のフラグがすべて設定されているかチェック
    pub fn contains_all(&self, flags: u16) -> bool {
        self.0 & flags == flags
    }

    /// 複数のフラグのいずれかが設定されているかチェック
    pub fn contains_any(&self, flags: u16) -> bool {
        self.0 & flags != 0
    }

    /// すべてのフラグをクリア
    pub fn clear(&mut self) {
        self.0 = 0;
    }

    /// 圧縮フラグが設定されているかチェック
    pub fn is_compressed(&self) -> bool {
        self.contains(Self::COMPRESSED)
    }

    /// 暗号化フラグが設定されているかチェック
    pub fn is_encrypted(&self) -> bool {
        self.contains(Self::ENCRYPTED)
    }

    /// 分割パケットかチェック
    pub fn is_fragmented(&self) -> bool {
        self.contains(Self::FRAGMENTED)
    }

    /// 最後の分割パケットかチェック
    pub fn is_last_fragment(&self) -> bool {
        self.contains(Self::LAST_FRAGMENT)
    }

    /// 高優先度パケットかチェック
    pub fn is_high_priority(&self) -> bool {
        self.contains(Self::PRIORITY_HIGH)
    }

    /// ACKが必要かチェック
    pub fn requires_ack(&self) -> bool {
        self.contains(Self::REQUIRES_ACK)
    }

    /// ACKパケットかチェック
    pub fn is_ack(&self) -> bool {
        self.contains(Self::IS_ACK)
    }

    /// キープアライブパケットかチェック
    pub fn is_keepalive(&self) -> bool {
        self.contains(Self::KEEPALIVE)
    }

    /// エラーパケットかチェック
    pub fn is_error(&self) -> bool {
        self.contains(Self::ERROR)
    }

    /// メタデータ付きかチェック
    pub fn has_metadata(&self) -> bool {
        self.contains(Self::METADATA)
    }

    /// チェックサム付きかチェック
    pub fn has_checksum(&self) -> bool {
        self.contains(Self::CHECKSUM)
    }
}

impl fmt::Display for FrameFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = Vec::new();

        if self.is_compressed() {
            flags.push("COMPRESSED");
        }
        if self.is_encrypted() {
            flags.push("ENCRYPTED");
        }
        if self.is_fragmented() {
            flags.push("FRAGMENTED");
        }
        if self.is_last_fragment() {
            flags.push("LAST_FRAGMENT");
        }
        if self.is_high_priority() {
            flags.push("PRIORITY_HIGH");
        }
        if self.requires_ack() {
            flags.push("REQUIRES_ACK");
        }
        if self.is_ack() {
            flags.push("IS_ACK");
        }
        if self.is_keepalive() {
            flags.push("KEEPALIVE");
        }
        if self.is_error() {
            flags.push("ERROR");
        }
        if self.has_metadata() {
            flags.push("METADATA");
        }
        if self.has_checksum() {
            flags.push("CHECKSUM");
        }

        if flags.is_empty() {
            write!(f, "FrameFlags(NONE)")
        } else {
            write!(f, "FrameFlags({})", flags.join(" | "))
        }
    }
}

impl From<u16> for FrameFlags {
    fn from(bits: u16) -> Self {
        Self(bits)
    }
}

impl From<FrameFlags> for u16 {
    fn from(flags: FrameFlags) -> Self {
        flags.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_operations() {
        let mut flags = FrameFlags::new();
        assert_eq!(flags.bits(), 0);

        // フラグを設定
        flags.set(FrameFlags::COMPRESSED);
        assert!(flags.is_compressed());
        assert!(!flags.is_encrypted());

        // 複数のフラグを設定
        flags.set(FrameFlags::PRIORITY_HIGH | FrameFlags::REQUIRES_ACK);
        assert!(flags.is_compressed());
        assert!(flags.is_high_priority());
        assert!(flags.requires_ack());

        // フラグをクリア
        flags.unset(FrameFlags::COMPRESSED);
        assert!(!flags.is_compressed());
        assert!(flags.is_high_priority());

        // すべてクリア
        flags.clear();
        assert_eq!(flags.bits(), 0);
    }

    #[test]
    fn test_contains_methods() {
        let mut flags = FrameFlags::new();
        flags.set(FrameFlags::COMPRESSED | FrameFlags::PRIORITY_HIGH);

        // 単一フラグチェック
        assert!(flags.contains(FrameFlags::COMPRESSED));
        assert!(!flags.contains(FrameFlags::ENCRYPTED));

        // 複数フラグチェック
        assert!(flags.contains_all(FrameFlags::COMPRESSED | FrameFlags::PRIORITY_HIGH));
        assert!(!flags.contains_all(FrameFlags::COMPRESSED | FrameFlags::ENCRYPTED));
        assert!(flags.contains_any(FrameFlags::COMPRESSED | FrameFlags::ENCRYPTED));
        assert!(!flags.contains_any(FrameFlags::ENCRYPTED | FrameFlags::FRAGMENTED));
    }

    #[test]
    fn test_display() {
        let mut flags = FrameFlags::new();
        assert_eq!(format!("{}", flags), "FrameFlags(NONE)");

        flags.set(FrameFlags::COMPRESSED | FrameFlags::PRIORITY_HIGH);
        let display = format!("{}", flags);
        assert!(display.contains("COMPRESSED"));
        assert!(display.contains("PRIORITY_HIGH"));
    }
}
