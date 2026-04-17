use std::collections::HashMap;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AuthResults {
    pub spf: Option<String>,
    pub dkim: Option<String>,
    pub dmarc: Option<String>,
    pub arc: Option<String>,
    pub raw: String,
}

pub struct HeaderParser;

impl HeaderParser {
    pub fn parse_auth_results(headers: &HashMap<String, String>) -> AuthResults {
        let raw = headers
            .iter()
            .filter(|(k, _)| k.to_lowercase() == "authentication-results")
            .map(|(_, v)| v.as_str())
            .collect::<Vec<_>>()
            .join("; ");

        AuthResults {
            spf: Self::extract_result(&raw, "spf"),
            dkim: Self::extract_result(&raw, "dkim"),
            dmarc: Self::extract_result(&raw, "dmarc"),
            arc: Self::extract_result(&raw, "arc"),
            raw: raw.clone(),
        }
    }

    fn extract_result(haystack: &str, protocol: &str) -> Option<String> {
        let lower = haystack.to_lowercase();
        let pattern = format!("{}=", protocol);
        let pos = lower.find(&pattern)?;
        let rest = &haystack[pos + pattern.len()..];
        let result = rest
            .split(|c: char| c == ';' || c == ' ' || c == '\n' || c == '\r')
            .next()?
            .trim()
            .to_string();
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    pub fn extract_return_path(headers: &HashMap<String, String>) -> Option<String> {
        headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == "return-path")
            .map(|(_, v)| v.clone())
    }

    pub fn extract_list_unsubscribe(headers: &HashMap<String, String>) -> Option<String> {
        headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == "list-unsubscribe")
            .map(|(_, v)| v.clone())
    }

    pub fn has_one_click_unsubscribe(headers: &HashMap<String, String>) -> bool {
        headers
            .iter()
            .any(|(k, v)| {
                k.to_lowercase() == "list-unsubscribe-post"
                    && v.to_lowercase().contains("list-unsubscribe=one-click")
            })
    }

    pub fn extract_message_id(headers: &HashMap<String, String>) -> Option<String> {
        headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == "message-id")
            .map(|(_, v)| v.trim_matches(|c| c == '<' || c == '>').to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_spf_pass() {
        let mut headers = HashMap::new();
        headers.insert(
            "Authentication-Results".into(),
            "mx.google.com; spf=pass (sender IP is 1.2.3.4) smtp.mailfrom=example.com; dkim=pass header.i=@example.com; dmarc=pass".into(),
        );
        let auth = HeaderParser::parse_auth_results(&headers);
        assert_eq!(auth.spf.as_deref(), Some("pass"));
        assert_eq!(auth.dkim.as_deref(), Some("pass"));
        assert_eq!(auth.dmarc.as_deref(), Some("pass"));
    }

    #[test]
    fn parses_dkim_fail() {
        let mut headers = HashMap::new();
        headers.insert(
            "authentication-results".into(),
            "mx.example.com; dkim=fail reason=\"signature verification failed\"".into(),
        );
        let auth = HeaderParser::parse_auth_results(&headers);
        assert_eq!(auth.dkim.as_deref(), Some("fail"));
    }
}
