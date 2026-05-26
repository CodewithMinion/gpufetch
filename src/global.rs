//! Global logging state

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

static VERBOSE: AtomicBool = AtomicBool::new(false);
static LOG_LEVEL: AtomicU8 = AtomicU8::new(0);

pub const LOG_LEVEL_WARN: u8 = 1;
pub const LOG_LEVEL_INFO: u8 = 2;
pub const LOG_LEVEL_DEBUG: u8 = 3;

/// Initialize global state from CLI flags
pub fn init(verbose: bool) {
    VERBOSE.store(verbose, Ordering::SeqCst);
    let level = if verbose {
        LOG_LEVEL_DEBUG
    } else {
        LOG_LEVEL_INFO
    };
    LOG_LEVEL.store(level, Ordering::SeqCst);
}

pub fn error(msg: &str) {
    let _ = LOG_LEVEL.load(Ordering::SeqCst);
    eprintln!("\x1b[31m[ERROR]\x1b[0m: {msg}");
}

pub fn warn(msg: &str) {
    if LOG_LEVEL.load(Ordering::SeqCst) >= LOG_LEVEL_WARN {
        eprintln!("\x1b[33m[WARN]\x1b[0m: {msg}");
    }
}

pub fn info(msg: &str) {
    if LOG_LEVEL.load(Ordering::SeqCst) >= LOG_LEVEL_INFO {
        eprintln!("\x1b[34m[INFO]\x1b[0m: {msg}");
    }
}

pub fn debug(msg: &str) {
    if LOG_LEVEL.load(Ordering::SeqCst) >= LOG_LEVEL_DEBUG {
        eprintln!("\x1b[36m[DEBUG]\x1b[0m: {msg}");
    }
}
