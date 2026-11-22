//! The log of the plugin: lines at a level with the target
//! `chora_construct`, which the kit's subscriber writes to stderr. The
//! macros call a function: the macros of `tracing` count as branches in
//! the complexity lint of every function that logs.

use std::fmt::Arguments;

const TARGET: &str = "chora_construct";

pub(crate) fn error(args: Arguments<'_>) {
    tracing::error!(target: TARGET, "{args}");
}

pub(crate) fn warn(args: Arguments<'_>) {
    tracing::warn!(target: TARGET, "{args}");
}

pub(crate) fn info(args: Arguments<'_>) {
    tracing::info!(target: TARGET, "{args}");
}

macro_rules! log_error {
    ($($t:tt)*) => { $crate::log::error(format_args!($($t)*)) };
}
macro_rules! log_warn {
    ($($t:tt)*) => { $crate::log::warn(format_args!($($t)*)) };
}
macro_rules! log_info {
    ($($t:tt)*) => { $crate::log::info(format_args!($($t)*)) };
}

pub(crate) use {log_error, log_info, log_warn};
