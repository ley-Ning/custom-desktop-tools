//! 文本工具箱：Base64 编解码与 MD5/SHA 系列哈希。
//! 编解码以 UTF-8 为准，哈希输出小写 hex。

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

/// 对文本做哈希。algo ∈ md5 | sha1 | sha256 | sha512，输出小写 hex。
#[tauri::command]
pub fn text_hash(input: String, algo: String) -> Result<String, String> {
    let bytes = input.as_bytes();
    match algo.as_str() {
        "md5" => {
            use md5::Digest as _;
            Ok(hex::encode(Md5::digest(bytes)))
        }
        "sha1" => {
            use sha1::Digest as _;
            Ok(hex::encode(Sha1::digest(bytes)))
        }
        "sha256" => Ok(hex::encode(Sha256::digest(bytes))),
        "sha512" => Ok(hex::encode(Sha512::digest(bytes))),
        other => Err(format!(
            "不支持的哈希算法：{}（可选 md5 / sha1 / sha256 / sha512）",
            other
        )),
    }
}

/// Base64 编码（UTF-8 安全，标准字母表 + 填充）。
#[tauri::command]
pub fn text_base64_encode(input: String) -> String {
    BASE64_STANDARD.encode(input.as_bytes())
}

/// Base64 解码为 UTF-8 文本；忽略其中混入的空白字符（换行粘贴很常见），
/// 非法输入返回错误而非乱码。
#[tauri::command]
pub fn text_base64_decode(input: String) -> Result<String, String> {
    let compact: String = input
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect();
    let bytes = BASE64_STANDARD
        .decode(compact.as_bytes())
        .map_err(|e| format!("无效的 Base64：{}", e))?;
    String::from_utf8(bytes).map_err(|_| "解码结果不是有效的 UTF-8 文本".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_known_vectors() {
        // 标准测试向量（NIST RFC 1321 / FIPS 180 附件）
        assert_eq!(
            text_hash("abc".into(), "md5".into()).unwrap(),
            "900150983cd24fb0d6963f7d28e17f72"
        );
        assert_eq!(
            text_hash("abc".into(), "sha1".into()).unwrap(),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(
            text_hash("abc".into(), "sha256".into()).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            text_hash("abc".into(), "sha512".into()).unwrap(),
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
             2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        );
    }

    #[test]
    fn hash_empty_and_utf8() {
        assert_eq!(
            text_hash(String::new(), "md5".into()).unwrap(),
            "d41d8cd98f00b204e9800998ecf8427e"
        );
        // 中文与 emoji 的 UTF-8 字节参与哈希
        assert_eq!(
            text_hash("你好".into(), "md5".into()).unwrap(),
            "7eca689f0d3389d9dea66ae112e5cfd7"
        );
    }

    #[test]
    fn hash_unknown_algo_is_error() {
        assert!(text_hash("x".into(), "crc32".into()).is_err());
    }

    #[test]
    fn base64_roundtrip_ascii_and_utf8() {
        assert_eq!(text_base64_encode("hello".into()), "aGVsbG8=");
        assert_eq!(
            text_base64_decode("aGVsbG8=".into()).unwrap(),
            "hello"
        );
        let s = "你好，RuYi ✨";
        let enc = text_base64_encode(s.into());
        assert_eq!(text_base64_decode(enc).unwrap(), s);
    }

    #[test]
    fn base64_decode_rejects_garbage_and_binary() {
        assert!(text_base64_decode("不是base64!!".into()).is_err());
        // 合法 Base64 但解出非 UTF-8 字节序列（\xff\xfe）
        assert!(text_base64_decode("//4=".into()).is_err());
    }

    #[test]
    fn base64_decode_tolerates_whitespace() {
        assert_eq!(
            text_base64_decode("aGVs\nbG8=".into()).unwrap(),
            "hello"
        );
    }
}
