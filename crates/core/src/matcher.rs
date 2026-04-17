use regex::Regex;

pub struct MessageMatcher;

impl MessageMatcher {
    pub fn subject_contains_token(subject: &str, token: &str) -> bool {
        subject.contains(token)
    }

    pub fn body_contains_token(body: &str, token: &str) -> bool {
        body.contains(token)
    }

    pub fn extract_token_from_subject(subject: &str) -> Option<String> {
        let re = Regex::new(r"\[MS-\d{8}-[0-9A-F]{4}\]").ok()?;
        re.find(subject).map(|m| m.as_str().to_string())
    }

    pub fn matches_criteria(
        subject: &str,
        body: Option<&str>,
        headers: &std::collections::HashMap<String, String>,
        subject_token: &str,
        body_token: &str,
    ) -> MatchScore {
        // Priority 1: custom header
        if let Some(header_token) = headers.get("x-mailscope-token") {
            if header_token == subject_token {
                return MatchScore::Definitive;
            }
        }

        // Priority 2: exact subject token
        if Self::subject_contains_token(subject, subject_token) {
            return MatchScore::Strong;
        }

        // Priority 3: body token
        if let Some(body) = body {
            if Self::body_contains_token(body, body_token) {
                return MatchScore::Strong;
            }
        }

        // Priority 4: message-id correlation (caller handles)
        MatchScore::None
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum MatchScore {
    None,
    Weak,
    Strong,
    Definitive,
}

impl MatchScore {
    pub fn is_match(&self) -> bool {
        *self >= MatchScore::Strong
    }
}
