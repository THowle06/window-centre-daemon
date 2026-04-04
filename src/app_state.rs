use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

#[derive(Debug)]
pub struct AppState {
    pub seen_windows: Mutex<HashSet<isize>>,
    pub enabled: Mutex<bool>,
}

static APP_STATE: OnceLock<AppState> = OnceLock::new();

pub fn init() {
    APP_STATE
        .set(AppState {
            seen_windows: Mutex::new(HashSet::new()),
            enabled: Mutex::new(true),
        })
        .unwrap();
}

pub fn get() -> &'static AppState {
    APP_STATE.get().unwrap()
}
