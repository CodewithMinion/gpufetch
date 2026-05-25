//! Global state management with Context
//! No more global static state - use Context for thread-safe state

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

static VERBOSE: AtomicBool = AtomicBool::new(false);
static LOG_LEVEL: AtomicU8 = AtomicU8::new(0);

pub const LOG_LEVEL_ERROR: u8 = 0;
pub const LOG_LEVEL_WARN: u8 = 1;
pub const LOG_LEVEL_INFO: u8 = 2;
pub const LOG_LEVEL_DEBUG: u8 = 3;

/// Application context - thread-safe state
pub struct Context {
    verbose: bool,
    log_level: u8,
}

impl Context {
    pub fn new(verbose: bool) -> Self {
        let level = if verbose { LOG_LEVEL_DEBUG } else { LOG_LEVEL_INFO };
        Self {
            verbose,
            log_level: level,
        }
    }
    
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }
    
    pub fn log_level(&self) -> u8 {
        self.log_level
    }
}

/// Initialize global state (legacy API, use Context for new code)
pub fn init(verbose: bool) {
    VERBOSE.store(verbose, Ordering::SeqCst);
    let level = if verbose { LOG_LEVEL_DEBUG } else { LOG_LEVEL_INFO };
    LOG_LEVEL.store(level, Ordering::SeqCst);
}

pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::SeqCst)
}

pub fn set_log_level(level: u8) {
    LOG_LEVEL.store(level, Ordering::SeqCst);
}

pub fn get_log_level() -> u8 {
    LOG_LEVEL.load(Ordering::SeqCst)
}

pub fn error(msg: &str) {
    eprintln!("\x1b[31m[ERROR]\x1b[0m: {}", msg);
}

pub fn warn(msg: &str) {
    if LOG_LEVEL.load(Ordering::SeqCst) >= LOG_LEVEL_WARN {
        eprintln!("\x1b[33m[WARN]\x1b[0m: {}", msg);
    }
}

pub fn info(msg: &str) {
    if LOG_LEVEL.load(Ordering::SeqCst) >= LOG_LEVEL_INFO {
        eprintln!("\x1b[34m[INFO]\x1b[0m: {}", msg);
    }
}

pub fn debug(msg: &str) {
    if LOG_LEVEL.load(Ordering::SeqCst) >= LOG_LEVEL_DEBUG {
        eprintln!("\x1b[36m[DEBUG]\x1b[0m: {}", msg);
    }
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::global::error(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::global::warn(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::global::info(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::global::debug(&format!($($arg)*))
    };
}
