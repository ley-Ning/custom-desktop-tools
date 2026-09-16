//! 聚合翻译插件后端
//!
//! 引擎：
//! - mymemory（默认）：https://api.mymemory.translated.net 免 key，支持 Autodetect，
//!   单次请求上限 500 字节，匿名额度约 5000 字符/天
//! - google：translate.googleapis.com gtx 端点，免 key，部分地区不可达
//! - ai：由前端复用 ai_chat::send_ai_message 流式通道，不走本模块

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::command;

/// MyMemory 单次请求的字节上限（官方限制 500 bytes）
const MYMEMORY_MAX_BYTES: usize = 500;

#[derive(Debug, Clone, Serialize)]
pub struct TranslateResult {
    pub text: String,
    /// 检测到的源语言（如 "en"、"zh-CN"）
    pub detected: Option<String>,
    pub engine: String,
}

#[derive(Debug, Deserialize)]
struct MyMemoryResponse {
    #[serde(default)]
    responseData: Option<MyMemoryResponseData>,
    #[serde(default)]
    responseStatus: i64,
    #[serde(default)]
    responseDetails: String,
}

#[derive(Debug, Deserialize)]
struct MyMemoryResponseData {
    #[serde(default)]
    translatedText: String,
    #[serde(default)]
    detectedLanguage: Option<String>,
}

/// 解析 Google gtx 响应：[[["译文","原文",...],...],null,"检测语言",...]
fn parse_google_response(value: &serde_json::Value) -> Option<(String, Option<String>)> {
    let segments = value.as_array()?.first()?.as_array()?;
    let mut text = String::new();
    for seg in segments {
        if let Some(t) = seg.as_array().and_then(|a| a.first()).and_then(|v| v.as_str()) {
            text.push_str(t);
        }
    }
    if text.is_empty() {
        return None;
    }
    let detected = value
        .as_array()
        .and_then(|a| a.get(2))
        .and_then(|v| v.as_str())
        .map(String::from);
    Some((text, detected))
}

/// 解析 MyMemory 响应
fn parse_mymemory_response(resp: &MyMemoryResponse) -> Result<TranslateResult, String> {
    if resp.responseStatus != 200 {
        return Err(format!(
            "MyMemory 错误 {}: {}",
            resp.responseStatus, resp.responseDetails
        ));
    }
    let data = resp
        .responseData
        .as_ref()
        .ok_or("MyMemory 返回数据为空")?;
    let text = data.translatedText.trim().to_string();
    // 免费额度用尽时 translatedText 会以警告文本代替译文
    if text.is_empty() || text.starts_with("MYMEMORY WARNING") || text.starts_with("PLEASE SELECT") {
        return Err(format!("MyMemory: {}", text));
    }
    Ok(TranslateResult {
        text,
        detected: data.detectedLanguage.clone(),
        engine: "mymemory".to_string(),
    })
}

/// 翻译文本
#[command]
pub async fn translate_text(
    text: String,
    from: String,
    to: String,
    engine: String,
) -> Result<TranslateResult, String> {
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("翻译内容为空".to_string());
    }
    if to == "auto" {
        return Err("目标语言不能是自动检测".to_string());
    }

    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    match engine.as_str() {
        "mymemory" => translate_mymemory(&client, &text, &from, &to).await,
        "google" => translate_google(&client, &text, &from, &to).await,
        other => Err(format!("不支持的引擎: {}", other)),
    }
}

async fn translate_mymemory(
    client: &reqwest::Client,
    text: &str,
    from: &str,
    to: &str,
) -> Result<TranslateResult, String> {
    if text.len() > MYMEMORY_MAX_BYTES {
        return Err(format!(
            "MyMemory 单次最多翻译 {} 字节（当前 {} 字节），长文本请切换 AI 翻译",
            MYMEMORY_MAX_BYTES,
            text.len()
        ));
    }

    let source = if from == "auto" { "Autodetect" } else { from };
    let langpair = format!("{}|{}", source, to);

    let resp = client
        .get("https://api.mymemory.translated.net/get")
        .query(&[("q", text), ("langpair", langpair.as_str())])
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
        .send()
        .await
        .map_err(|e| format!("MyMemory 请求失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("MyMemory HTTP 错误: {}", resp.status()));
    }

    let body = resp
        .json::<MyMemoryResponse>()
        .await
        .map_err(|e| format!("解析 MyMemory 响应失败: {}", e))?;
    parse_mymemory_response(&body)
}

async fn translate_google(
    client: &reqwest::Client,
    text: &str,
    from: &str,
    to: &str,
) -> Result<TranslateResult, String> {
    let resp = client
        .get("https://translate.googleapis.com/translate_a/single")
        .query(&[
            ("client", "gtx"),
            ("sl", from),
            ("tl", to),
            ("dt", "t"),
            ("q", text),
        ])
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
        .send()
        .await
        .map_err(|e| format!("Google 请求失败（该服务在部分地区不可达）: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Google HTTP 错误: {}", resp.status()));
    }

    let value: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析 Google 响应失败: {}", e))?;

    let (text, detected) =
        parse_google_response(&value).ok_or("Google 返回内容无法解析".to_string())?;
    Ok(TranslateResult {
        text,
        detected,
        engine: "google".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_google_single_segment() {
        let json: serde_json::Value = serde_json::from_str(
            r#"[[["你好","Hello",null,null,3]],null,"en"]"#,
        )
        .unwrap();
        let (text, detected) = parse_google_response(&json).unwrap();
        assert_eq!(text, "你好");
        assert_eq!(detected.as_deref(), Some("en"));
    }

    #[test]
    fn test_parse_google_multi_segment() {
        let json: serde_json::Value = serde_json::from_str(
            r#"[[["你好","Hello",null,null,3],["世界","world",null,null,4]],null,"en"]"#,
        )
        .unwrap();
        let (text, detected) = parse_google_response(&json).unwrap();
        assert_eq!(text, "你好世界");
        assert_eq!(detected.as_deref(), Some("en"));
    }

    #[test]
    fn test_parse_google_invalid() {
        let json: serde_json::Value = serde_json::from_str(r#"[]"#).unwrap();
        assert!(parse_google_response(&json).is_none());
        let json: serde_json::Value = serde_json::from_str(r#"[[[]],null,"en"]"#).unwrap();
        assert!(parse_google_response(&json).is_none());
    }

    #[test]
    fn test_parse_mymemory_ok_with_detected() {
        let body = r#"{"responseData":{"translatedText":"Hello world","match":0.85,"detectedLanguage":"zh-CN"},"responseStatus":200,"responseDetails":""}"#;
        let resp: MyMemoryResponse = serde_json::from_str(body).unwrap();
        let result = parse_mymemory_response(&resp).unwrap();
        assert_eq!(result.text, "Hello world");
        assert_eq!(result.detected.as_deref(), Some("zh-CN"));
        assert_eq!(result.engine, "mymemory");
    }

    #[test]
    fn test_parse_mymemory_ok_without_detected() {
        let body = r#"{"responseData":{"translatedText":"你好世界","match":1},"responseStatus":200,"responseDetails":""}"#;
        let resp: MyMemoryResponse = serde_json::from_str(body).unwrap();
        let result = parse_mymemory_response(&resp).unwrap();
        assert_eq!(result.text, "你好世界");
        assert!(result.detected.is_none());
    }

    #[test]
    fn test_parse_mymemory_quota_warning() {
        let body = r#"{"responseData":{"translatedText":"MYMEMORY WARNING: YOU USED ALL AVAILABLE FREE TRANSLATIONS FOR TODAY"},"responseStatus":200,"responseDetails":""}"#;
        let resp: MyMemoryResponse = serde_json::from_str(body).unwrap();
        assert!(parse_mymemory_response(&resp).is_err());
    }

    #[test]
    fn test_parse_mymemory_error_status() {
        let body = r#"{"responseData":null,"responseStatus":429,"responseDetails":"Too many requests"}"#;
        let resp: MyMemoryResponse = serde_json::from_str(body).unwrap();
        let err = parse_mymemory_response(&resp).unwrap_err();
        assert!(err.contains("429"));
    }

    #[test]
    fn test_mymemory_length_limit() {
        let long_text = "a".repeat(MYMEMORY_MAX_BYTES + 1);
        assert!(long_text.len() > MYMEMORY_MAX_BYTES);
        // 命令层在异步环境里校验，这里直接验证阈值逻辑的数据前提
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let err = rt
            .block_on(translate_text(long_text, "auto".into(), "en".into(), "mymemory".into()))
            .unwrap_err();
        assert!(err.contains("500"));
    }

    #[test]
    fn test_translate_text_empty_rejected() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        assert!(rt
            .block_on(translate_text("  ".into(), "auto".into(), "en".into(), "mymemory".into()))
            .is_err());
        // 目标语言不能为 auto
        assert!(rt
            .block_on(translate_text("hi".into(), "auto".into(), "auto".into(), "mymemory".into()))
            .is_err());
        // 未知引擎
        assert!(rt
            .block_on(translate_text("hi".into(), "auto".into(), "en".into(), "deepl".into()))
            .is_err());
    }
}
