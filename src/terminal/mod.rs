use chrono::{DateTime, Local};

pub mod question;
pub mod shell;
pub mod logger;

pub fn timestamp() -> String {
    let now: DateTime<Local> = Local::now();
    format!("[{:.11}]", now.format("%H:%M:%S%.3f"))
}