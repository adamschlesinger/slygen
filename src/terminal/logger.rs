use std::io::stdout;
use std::sync::OnceLock;

use clap::ValueEnum;
use crossterm::execute;
use crossterm::style::{Color, Print, SetForegroundColor};
use serde::{Deserialize, Serialize};
use crate::terminal::timestamp;

#[macro_export]
macro_rules! header {
    ($($arg:tt)*) => {{
        $crate::terminal::logger::__header(format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        $crate::terminal::logger::__log($crate::terminal::logger::LogLevel::Debug, format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        $crate::terminal::logger::__log($crate::terminal::logger::LogLevel::Info, format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::terminal::logger::__log($crate::terminal::logger::LogLevel::Warn, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::terminal::logger::__log($crate::terminal::logger::LogLevel::Error, format!($($arg)*))
    };
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Panic,
    None,
}

/// Sets the LogLevel
pub fn init(level: LogLevel) {
    match LOG_LEVEL.set(level) {
        Ok(_) | Err(_) => {} // no op
    }
}

/// todo
#[doc(hidden)]
pub fn __log(level: LogLevel, msg: String) {
    match LOG_LEVEL.get() {
        None => {} // not yet init - just ignore
        Some(logger_level) => {
            if level >= *logger_level {
                let _ = execute!(
                    stdout(),
                    SetForegroundColor(Color::Grey),
                    Print(timestamp()),
                    SetForegroundColor(color(&level)),
                    Print(format!(" {msg}\n")),
                );
            }
        }
    }
}

#[doc(hidden)]
pub fn __header(msg: String) {
    let msg = format!("=== {msg} ===");
    let dashes: String = msg.chars().map(|_| '_').collect();
    let _ = execute!(
        stdout(),
        SetForegroundColor(Color::Grey),
        Print(timestamp()),
        Print("\n"),
        Print(timestamp()),
        SetForegroundColor(Color::Green),
        Print(format!(" {msg}\n")),
        SetForegroundColor(Color::Grey),
        Print(timestamp()),
        SetForegroundColor(Color::Green),
        Print(format!(" {dashes}\n")),
    );
}

#[doc(hidden)]
pub fn __debug(msg: String) {
    let _ = execute!(
        stdout(),
        SetForegroundColor(Color::Grey),
        Print(timestamp()),
        Print(format!(" {msg}\n")),
    );
}

static LOG_LEVEL: OnceLock<LogLevel> = OnceLock::new();

fn token(level: &LogLevel) -> &str {
    match level {
        LogLevel::Debug => "D",
        LogLevel::Info => "I",
        LogLevel::Warn => "W",
        LogLevel::Error => "X",
        LogLevel::Panic => "!",
        LogLevel::None => " ",
    }
}

fn color(level: &LogLevel) -> Color {
    match level {
        LogLevel::Debug => Color::Grey,
        LogLevel::Info => Color::White,
        LogLevel::Warn => Color::Yellow,
        LogLevel::Error => Color::Red,
        LogLevel::Panic => Color::Magenta,
        LogLevel::None => Color::Grey,
    }
}
