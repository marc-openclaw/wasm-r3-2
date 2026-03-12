//! Logger module for WASM - bridges Rust logs to JavaScript

use wasm_bindgen::prelude::*;

/// Info level log
pub fn info(msg: &str) {
    web_sys::console::log_1(&format!("[WASM] {}", msg).into());
}

/// Warning level log
pub fn warn_msg(msg: &str) {
    web_sys::console::warn_1(&format!("[WASM] {}", msg).into());
}

/// Error level log
pub fn error(msg: &str) {
    web_sys::console::error_1(&format!("[WASM] {}", msg).into());
}

/// Debug level log (only when verbose)
static mut VERBOSE: bool = false;

pub fn set_verbose(enabled: bool) {
    unsafe { VERBOSE = enabled; }
}

pub fn is_verbose() -> bool {
    unsafe { VERBOSE }
}

pub fn debug(msg: &str) {
    if is_verbose() {
        web_sys::console::log_1(&format!("[WASM:DEBUG] {}", msg).into());
    }
}

/// Initialize logger from JavaScript
#[wasm_bindgen(js_name = initLogger)]
pub fn init_logger(verbose: bool) {
    set_verbose(verbose);
}
