//! パケットのシリアライゼーション/デシリアライゼーション
//!
//! UnisonPacketとBytesの相互変換、圧縮/解凍処理を実装します。

use bytes::{BufMut, Bytes, BytesMut};
use crc32fast::Hasher as Crc32Hasher;
use rkyv::Deserialize;
use thiserror::Error;
use zstd::stream::{decode_all, encode_all};

use super::{
    flags::PacketFlags,
    header::UnisonPacketHeader,
    payload::{PayloadError, Payloadable},
};

/// 圧縮閾値: 2KB以上のペイロードを自動圧縮
pub const COMPRESSION_THRESHOLD: usize = 2048;

/// 圧縮レベル: 1（最速）
pub const COMPRESSION_LEVEL: i32 = 1;

/// 最大パケットサイズ: 16MB
pub const MAX_PACKET_SIZE: usize = 16 * 1024 * 1024;

/// シリアライゼーションエラー
#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("Payload error: {0}")]
    Payload(#[from] PayloadError),

    #[error("Compression failed: {0}")]
    CompressionFailed(String),

    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),

    #[error("Packet too large: {size} bytes (max: {max_size} bytes)")]
    PacketTooLarge { size: usize, max_size: usize },

    #[error("Invalid header")]
    InvalidHeader,

    #[error("Checksum mismatch: expected {expected:#010x}, got {actual:#010x}")]
    ChecksumMismatch { expected: u32, actual: u32 },

    #[error("Incompatible protocol version: {version}")]
    IncompatibleVersion { version: u8 },

    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
}

/// パケットのシリアライゼーション処理
pub struct PacketSerializer;

impl PacketSerializer {
    /// ヘッダーとペイロードをBytesに変換
    pub fn serialize<T: Payloadable>(
        header: &mut UnisonPacketHeader,
        payload: &T,
    ) -> Result<Bytes, SerializationError> {
        // ペイロードをシリアライズ
        let payload_bytes = payload.to_bytes()?;
        let payload_size = payload_bytes.len();

        // ペイロードサイズを設定
        header.payload_length = payload_size as u32;

        // 圧縮判定と処理
        let (final_payload, is_compressed) = if payload_size >= COMPRESSION_THRESHOLD {
            let compressed = Self::compress(&payload_bytes)?;
            let compressed_size = compressed.len();

            // 圧縮が効果的な場合のみ使用
            if compressed_size < payload_size {
                header.compressed_length = compressed_size as u32;
                (compressed, true)
            } else {
                header.compressed_length = 0;
                (payload_bytes, false)
            }
        } else {
            header.compressed_length = 0;
            (payload_bytes, false)
        };

        // フラグを更新
        let mut flags = header.flags();
        if is_compressed {
            flags.set(PacketFlags::COMPRESSED);
        } else {
            flags.unset(PacketFlags::COMPRESSED);
        }
        header.set_flags(flags);

        // チェックサム計算（必要な場合）
        if header.checksum != 0 {
            header.checksum = Self::calculate_checksum(&final_payload);
        }

        // ヘッダーをシリアライズ
        let header_bytes = Self::serialize_header(header)?;

        // 最終的なパケットを構築
        let total_size = header_bytes.len() + final_payload.len();
        if total_size > MAX_PACKET_SIZE {
            return Err(SerializationError::PacketTooLarge {
                size: total_size,
                max_size: MAX_PACKET_SIZE,
            });
        }

        let mut packet = BytesMut::with_capacity(total_size);
        packet.put(header_bytes);
        packet.put(final_payload);

        Ok(packet.freeze())
    }

    /// ヘッダーをシリアライズ
    fn serialize_header(header: &UnisonPacketHeader) -> Result<Bytes, SerializationError> {
        let bytes = rkyv::to_bytes::<_, 256>(header)
            .map_err(|e| SerializationError::SerializationFailed(e.to_string()))?;
        Ok(Bytes::from(bytes.to_vec()))
    }

    /// ペイロードを圧縮
    fn compress(data: &[u8]) -> Result<Bytes, SerializationError> {
        encode_all(data, COMPRESSION_LEVEL)
            .map(Bytes::from)
            .map_err(|e| SerializationError::CompressionFailed(e.to_string()))
    }

    /// CRC32チェックサムを計算
    fn calculate_checksum(data: &[u8]) -> u32 {
        let mut hasher = Crc32Hasher::new();
        hasher.update(data);
        hasher.finalize()
    }
}

/// パケットのデシリアライゼーション処理
pub struct PacketDeserializer;

impl PacketDeserializer {
    /// Bytesからヘッダーとペイロードを分離
    pub fn deserialize_header(
        bytes: &Bytes,
    ) -> Result<(UnisonPacketHeader, Bytes), SerializationError> {
        if bytes.len() < 48 {
            return Err(SerializationError::InvalidHeader);
        }

        // ヘッダー部分を取得（最初の48バイト）
        let header_bytes = &bytes[..48];
        let header = Self::parse_header(header_bytes)?;

        // バージョンチェック
        if !header.is_compatible() {
            return Err(SerializationError::IncompatibleVersion {
                version: header.version,
            });
        }

        // ペイロード部分を取得
        let payload_bytes = bytes.slice(48..);

        Ok((header, payload_bytes))
    }

    /// ペイロードをデシリアライズ
    pub fn deserialize_payload<T: Payloadable>(
        header: &UnisonPacketHeader,
        payload_bytes: &Bytes,
    ) -> Result<T, SerializationError>
    where
        T::Archived: Deserialize<T, rkyv::Infallible>,
        for<'a> T::Archived: rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'a>>,
    {
        // サイズチェック
        let expected_size = header.actual_payload_size() as usize;
        if payload_bytes.len() != expected_size {
            return Err(SerializationError::InvalidHeader);
        }

        // チェックサム検証（必要な場合）
        if header.has_checksum() {
            let calculated = PacketSerializer::calculate_checksum(payload_bytes);
            if calculated != header.checksum {
                return Err(SerializationError::ChecksumMismatch {
                    expected: header.checksum,
                    actual: calculated,
                });
            }
        }

        // 解凍（必要な場合）
        let decompressed = if header.is_compressed() {
            Self::decompress(payload_bytes)?
        } else {
            payload_bytes.clone()
        };

        // ペイロードをデシリアライズ
        T::from_bytes(&decompressed).map_err(Into::into)
    }

    /// ゼロコピーでペイロードの参照を取得
    pub fn deserialize_payload_zero_copy<'a, T: Payloadable>(
        header: &UnisonPacketHeader,
        payload_bytes: &'a [u8],
        buffer: &'a mut Vec<u8>,
    ) -> Result<&'a T::Archived, SerializationError>
    where
        for<'b> T::Archived: rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'b>>,
    {
        // チェックサム検証（必要な場合）
        if header.has_checksum() {
            let calculated = PacketSerializer::calculate_checksum(payload_bytes);
            if calculated != header.checksum {
                return Err(SerializationError::ChecksumMismatch {
                    expected: header.checksum,
                    actual: calculated,
                });
            }
        }

        // 解凍が必要な場合はバッファを使用
        if header.is_compressed() {
            *buffer = Self::decompress_to_vec(payload_bytes)?;
            T::from_bytes_zero_copy(buffer).map_err(Into::into)
        } else {
            // 圧縮されていない場合は直接ゼロコピー
            T::from_bytes_zero_copy(payload_bytes).map_err(Into::into)
        }
    }

    /// ヘッダーをパース
    fn parse_header(bytes: &[u8]) -> Result<UnisonPacketHeader, SerializationError> {
        let archived = rkyv::check_archived_root::<UnisonPacketHeader>(bytes)
            .map_err(|e| SerializationError::DeserializationFailed(e.to_string()))?;

        archived
            .deserialize(&mut rkyv::Infallible)
            .map_err(|_| SerializationError::InvalidHeader)
    }

    /// データを解凍
    fn decompress(data: &[u8]) -> Result<Bytes, SerializationError> {
        decode_all(data)
            .map(Bytes::from)
            .map_err(|e| SerializationError::DecompressionFailed(e.to_string()))
    }

    /// データを解凍（Vec<u8>として）
    fn decompress_to_vec(data: &[u8]) -> Result<Vec<u8>, SerializationError> {
        decode_all(data).map_err(|e| SerializationError::DecompressionFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::PacketType;
    use crate::packet::payload::{BytesPayload, StringPayload};

    #[test]
    fn test_serialize_small_packet() {
        // 圧縮閾値未満のパケット
        let mut header = UnisonPacketHeader::new(PacketType::Data);
        let payload = StringPayload::from_str("Hello, World!");

        let packet = PacketSerializer::serialize(&mut header, &payload).unwrap();

        assert!(!header.is_compressed());
        assert_eq!(header.compressed_length, 0);
        assert!(packet.len() > 48); // ヘッダー + ペイロード
    }

    #[test]
    fn test_serialize_large_packet() {
        // 圧縮閾値以上のパケット
        let mut header = UnisonPacketHeader::new(PacketType::Data);
        let large_text = "x".repeat(3000);
        let payload = StringPayload::new(large_text);

        let packet = PacketSerializer::serialize(&mut header, &payload).unwrap();

        assert!(header.is_compressed());
        assert!(header.compressed_length > 0);
        assert!(header.compressed_length < header.payload_length);
    }

    #[test]
    fn test_round_trip() {
        let mut header = UnisonPacketHeader::new(PacketType::Data)
            .with_sequence(42)
            .with_stream_id(1337);
        header.checksum = 1; // チェックサムを有効化

        let payload = StringPayload::from_str("Test payload data");

        // シリアライズ
        let packet = PacketSerializer::serialize(&mut header, &payload).unwrap();

        // デシリアライズ
        let (restored_header, payload_bytes) =
            PacketDeserializer::deserialize_header(&packet).unwrap();
        let restored_payload: StringPayload =
            PacketDeserializer::deserialize_payload(&restored_header, &payload_bytes).unwrap();

        assert_eq!(restored_header.sequence_number, 42);
        assert_eq!(restored_header.stream_id, 1337);
        assert_eq!(restored_payload.data, "Test payload data");
    }

    #[test]
    fn test_zero_copy_deserialization() {
        let mut header = UnisonPacketHeader::new(PacketType::Data);
        let payload = BytesPayload::new(vec![1, 2, 3, 4, 5]);

        let packet = PacketSerializer::serialize(&mut header, &payload).unwrap();
        let (restored_header, payload_bytes) =
            PacketDeserializer::deserialize_header(&packet).unwrap();

        let mut buffer = Vec::new();
        let archived = PacketDeserializer::deserialize_payload_zero_copy::<BytesPayload>(
            &restored_header,
            &payload_bytes,
            &mut buffer,
        )
        .unwrap();

        assert_eq!(archived.data.as_slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_checksum_validation() {
        let mut header = UnisonPacketHeader::new(PacketType::Data);
        header.checksum = 1; // チェックサムを有効化

        let payload = StringPayload::from_str("Test");
        let mut packet = PacketSerializer::serialize(&mut header, &payload).unwrap();

        // パケットを改ざん
        let last_idx = packet.len() - 1;
        let mut modified = packet.to_vec();
        modified[last_idx] ^= 0xFF;
        let corrupted = Bytes::from(modified);

        // デシリアライズ時にチェックサムエラーになるはず
        let (header, payload_bytes) = PacketDeserializer::deserialize_header(&corrupted).unwrap();
        let result: Result<StringPayload, _> =
            PacketDeserializer::deserialize_payload(&header, &payload_bytes);

        assert!(matches!(
            result,
            Err(SerializationError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn test_compression_effectiveness() {
        // 圧縮が効果的なデータ
        let mut header = UnisonPacketHeader::new(PacketType::Data);
        let repetitive_data = "a".repeat(3000);
        let payload = StringPayload::new(repetitive_data);

        let packet = PacketSerializer::serialize(&mut header, &payload).unwrap();

        assert!(header.is_compressed());
        assert!(header.compressed_length < header.payload_length / 2);
    }
}
