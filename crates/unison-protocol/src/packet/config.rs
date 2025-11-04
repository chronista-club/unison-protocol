//! フレーム処理の設定
//!
//! 圧縮やチェックサムなどのフレーム処理に関する設定を管理します。

use serde::{Deserialize, Serialize};

/// 圧縮に関する設定
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// 圧縮を適用する最小ペイロードサイズ（バイト）
    /// この値より小さいペイロードは圧縮されません
    pub threshold: usize,

    /// zstd圧縮レベル（1-22）
    /// - 1: 最速（圧縮率低）
    /// - 3: デフォルト
    /// - 22: 最高圧縮（処理遅い）
    pub level: i32,

    /// 圧縮を有効にするかどうか
    pub enabled: bool,
}

impl CompressionConfig {
    /// デフォルト設定で新しいCompressionConfigを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// カスタム設定でCompressionConfigを作成
    pub fn custom(threshold: usize, level: i32) -> Self {
        Self {
            threshold,
            level: level.clamp(1, 22),
            enabled: true,
        }
    }

    /// 圧縮を無効化した設定を作成
    pub fn disabled() -> Self {
        Self {
            threshold: usize::MAX,
            level: 1,
            enabled: false,
        }
    }

    /// 高速圧縮設定（レベル1、閾値2KB）
    pub fn fast() -> Self {
        Self {
            threshold: 2048,
            level: 1,
            enabled: true,
        }
    }

    /// バランス設定（レベル3、閾値4KB）
    pub fn balanced() -> Self {
        Self {
            threshold: 4096,
            level: 3,
            enabled: true,
        }
    }

    /// 高圧縮設定（レベル9、閾値1KB）
    pub fn high_compression() -> Self {
        Self {
            threshold: 1024,
            level: 9,
            enabled: true,
        }
    }

    /// ペイロードが圧縮対象かどうかを判定
    pub fn should_compress(&self, payload_size: usize) -> bool {
        self.enabled && payload_size >= self.threshold
    }
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            threshold: 2048, // 2KB
            level: 1,        // 最速圧縮
            enabled: true,
        }
    }
}

/// フレーム処理の統合設定
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketConfig {
    /// 圧縮設定
    pub compression: CompressionConfig,

    /// 最大ペイロードサイズ（バイト）
    pub max_payload_size: usize,

    /// フレームバージョン
    pub version: u8,
}

impl PacketConfig {
    /// デフォルト設定で新しいPacketConfigを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// ビルダーパターンで圧縮設定を変更
    pub fn with_compression(mut self, config: CompressionConfig) -> Self {
        self.compression = config;
        self
    }

    /// ビルダーパターンで最大ペイロードサイズを設定
    pub fn with_max_payload_size(mut self, size: usize) -> Self {
        self.max_payload_size = size;
        self
    }

    /// 高性能設定（圧縮無効）
    pub fn high_performance() -> Self {
        Self {
            compression: CompressionConfig::disabled(),
            max_payload_size: 16 * 1024 * 1024, // 16MB
            version: 1,
        }
    }

    /// バランス設定（圧縮有効）
    pub fn balanced() -> Self {
        Self {
            compression: CompressionConfig::balanced(),
            max_payload_size: 16 * 1024 * 1024, // 16MB
            version: 1,
        }
    }

    /// 低帯域幅設定（高圧縮）
    pub fn low_bandwidth() -> Self {
        Self {
            compression: CompressionConfig::high_compression(),
            max_payload_size: 4 * 1024 * 1024, // 4MB
            version: 1,
        }
    }
}

impl Default for PacketConfig {
    fn default() -> Self {
        Self {
            compression: CompressionConfig::default(),
            max_payload_size: 16 * 1024 * 1024, // 16MB
            version: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_config_default() {
        let config = CompressionConfig::default();
        assert_eq!(config.threshold, 2048);
        assert_eq!(config.level, 1);
        assert!(config.enabled);
    }

    #[test]
    fn test_compression_config_presets() {
        let fast = CompressionConfig::fast();
        assert_eq!(fast.level, 1);
        assert_eq!(fast.threshold, 2048);

        let balanced = CompressionConfig::balanced();
        assert_eq!(balanced.level, 3);
        assert_eq!(balanced.threshold, 4096);

        let high = CompressionConfig::high_compression();
        assert_eq!(high.level, 9);
        assert_eq!(high.threshold, 1024);
    }

    #[test]
    fn test_should_compress() {
        let config = CompressionConfig::default();

        assert!(!config.should_compress(1024)); // 閾値未満
        assert!(config.should_compress(2048)); // 閾値と同じ
        assert!(config.should_compress(4096)); // 閾値より大きい

        let disabled = CompressionConfig::disabled();
        assert!(!disabled.should_compress(10000)); // 無効化されている
    }

    #[test]
    fn test_compression_level_clamp() {
        let config = CompressionConfig::custom(1024, 100);
        assert_eq!(config.level, 22); // 最大値にクランプ

        let config = CompressionConfig::custom(1024, -5);
        assert_eq!(config.level, 1); // 最小値にクランプ
    }

    #[test]
    fn test_packet_config_presets() {
        let perf = PacketConfig::high_performance();
        assert!(!perf.compression.enabled);

        let balanced = PacketConfig::balanced();
        assert!(balanced.compression.enabled);

        let low_bw = PacketConfig::low_bandwidth();
        assert_eq!(low_bw.compression.level, 9);
    }

    #[test]
    fn test_packet_config_builder() {
        let config = PacketConfig::new()
            .with_compression(CompressionConfig::fast())
            .with_max_payload_size(1024 * 1024);

        assert_eq!(config.compression.level, 1);
        assert_eq!(config.max_payload_size, 1024 * 1024);
    }
}
