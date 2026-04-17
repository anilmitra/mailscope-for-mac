use chrono::Utc;
use uuid::Uuid;

pub fn generate_token() -> String {
    let date = Utc::now().format("%Y%m%d").to_string();
    let suffix = &Uuid::new_v4().to_string().to_uppercase().replace('-', "")[..4];
    format!("MS-{}-{}", date, suffix)
}
