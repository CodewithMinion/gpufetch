//! Shared utilities

use std::io::ErrorKind;
use std::process::{Command, Output};

#[cfg(target_os = "macos")]
use std::path::Path;

/// Runs a subprocess. Returns `None` when the executable is missing (ENOENT).
pub fn try_output(command: &str, args: &[&str]) -> Option<Output> {
    match Command::new(command).args(args).output() {
        Ok(output) => Some(output),
        Err(e) if e.kind() == ErrorKind::NotFound => None,
        Err(e) => {
            crate::debug(&format!("command `{command}` failed to start: {e}"));
            None
        }
    }
}

/// Path to `system_profiler` on macOS (may be outside minimal `PATH`).
#[cfg(target_os = "macos")]
pub fn system_profiler_bin() -> &'static str {
    const SYSTEM_PROFILER: &str = "/usr/sbin/system_profiler";
    if Path::new(SYSTEM_PROFILER).exists() {
        SYSTEM_PROFILER
    } else {
        "system_profiler"
    }
}
