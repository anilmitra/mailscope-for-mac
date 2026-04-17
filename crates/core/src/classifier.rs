use crate::models::MatchedMessage;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Placement {
    Inbox,
    Spam,
    Junk,
    Promotions,
    Social,
    Updates,
    Missing,
    Other,
}

impl std::fmt::Display for Placement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Placement::Inbox => "inbox",
            Placement::Spam => "spam",
            Placement::Junk => "junk",
            Placement::Promotions => "promotions",
            Placement::Social => "social",
            Placement::Updates => "updates",
            Placement::Missing => "missing",
            Placement::Other => "other",
        };
        write!(f, "{}", s)
    }
}

pub struct PlacementClassifier;

impl PlacementClassifier {
    pub fn classify_gmail(msg: &MatchedMessage) -> Placement {
        let labels: Vec<&str> = msg.labels.iter().map(|s| s.as_str()).collect();

        if labels.contains(&"SPAM") {
            return Placement::Spam;
        }
        if labels.contains(&"TRASH") {
            return Placement::Other;
        }

        let in_inbox = labels.contains(&"INBOX");

        if labels.contains(&"CATEGORY_PROMOTIONS") {
            if in_inbox {
                return Placement::Promotions;
            }
        }
        if labels.contains(&"CATEGORY_SOCIAL") {
            if in_inbox {
                return Placement::Social;
            }
        }
        if labels.contains(&"CATEGORY_UPDATES") {
            if in_inbox {
                return Placement::Updates;
            }
        }
        if in_inbox {
            return Placement::Inbox;
        }

        Placement::Other
    }

    pub fn classify_microsoft(msg: &MatchedMessage) -> Placement {
        let folder = msg.folder.to_lowercase();
        match folder.as_str() {
            "inbox" => Placement::Inbox,
            "junkemail" | "junk email" | "spam" => Placement::Junk,
            "deleteditems" | "deleted items" | "trash" => Placement::Other,
            _ => Placement::Other,
        }
    }

    pub fn classify_imap(msg: &MatchedMessage) -> Placement {
        let folder = msg.folder.to_lowercase();
        if folder == "inbox" || folder == "inbo" {
            return Placement::Inbox;
        }
        if folder.contains("spam") || folder.contains("junk") || folder.contains("[gmail]/spam") {
            return Placement::Spam;
        }
        Placement::Other
    }

    pub fn classify(provider: &str, msg: &MatchedMessage) -> Placement {
        match provider {
            "gmail" => Self::classify_gmail(msg),
            "microsoft" => Self::classify_microsoft(msg),
            _ => Self::classify_imap(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MatchedMessage;
    use std::collections::HashMap;

    fn msg(folder: &str, labels: &[&str]) -> MatchedMessage {
        MatchedMessage {
            remote_id: "id".into(),
            folder: folder.into(),
            labels: labels.iter().map(|s| s.to_string()).collect(),
            headers: HashMap::new(),
            received_at: None,
        }
    }

    #[test]
    fn gmail_inbox() {
        assert_eq!(
            PlacementClassifier::classify_gmail(&msg("INBOX", &["INBOX", "UNREAD"])),
            Placement::Inbox
        );
    }

    #[test]
    fn gmail_spam() {
        assert_eq!(
            PlacementClassifier::classify_gmail(&msg("SPAM", &["SPAM"])),
            Placement::Spam
        );
    }

    #[test]
    fn gmail_promotions() {
        assert_eq!(
            PlacementClassifier::classify_gmail(&msg(
                "INBOX",
                &["INBOX", "CATEGORY_PROMOTIONS"]
            )),
            Placement::Promotions
        );
    }

    #[test]
    fn microsoft_junk() {
        assert_eq!(
            PlacementClassifier::classify_microsoft(&msg("JunkEmail", &[])),
            Placement::Junk
        );
    }
}
