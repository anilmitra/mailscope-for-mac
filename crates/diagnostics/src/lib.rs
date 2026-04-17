mod dns;
mod rules;

pub use dns::{check_domain, DomainCheckResult};
pub use rules::{run_diagnostics, DiagnosticResult};

pub fn lookup_provider(email: &str) -> &'static str {
    let domain = email.split('@').last().unwrap_or("").to_lowercase();
    match domain.as_str() {
        d if d.ends_with("gmail.com") || d.ends_with("googlemail.com") => "gmail",
        d if d.ends_with("outlook.com")
            || d.ends_with("hotmail.com")
            || d.ends_with("live.com")
            || d.ends_with("msn.com") =>
        {
            "microsoft"
        }
        _ => "imap",
    }
}
