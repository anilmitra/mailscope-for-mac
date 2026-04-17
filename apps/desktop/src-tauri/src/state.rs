use mailscope_storage::Database;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Database>,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self {
            db: Mutex::new(db),
        }
    }
}
