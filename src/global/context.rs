use std::{fs::File, sync::{OnceLock, RwLock}};

// TODO: Use a safer lock when possible.
pub static LOG_FILE: RwLock<OnceLock<File>> = RwLock::new(OnceLock::new());