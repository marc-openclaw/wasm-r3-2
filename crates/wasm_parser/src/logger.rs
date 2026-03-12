//! Logger module for WASM - bridges Rust logs to JavaScript

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Info level log
pub fn info(msg: &str) {
    #[cfg(feature = "wasm")]
    web_sys::console::log_1(&format!("[WASM] {}", msg).into());
    #[cfg(not(feature = "wasm"))]
    println!("[INFO] {}", msg);
}

/// Warning level log
pub fn warn_msg(msg: &str) {
    #[cfg(feature = "wasm")]
    web_sys::console::warn_1(&format!("[WASM] {}", msg).into());
    #[cfg(not(feature = "wasm"))]
    eprintln!("[WARN] {}", msg);
}

/// Error level log
pub fn error(msg: &str) {
    #[cfg(feature = "wasm")]
    web_sys::console::error_1(&format!("[WASM] {}", msg).into());
    #[cfg(not(feature = "wasm"))]
    eprintln!("[ERROR] {}", msg);
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
        #[cfg(feature = "wasm")]
        web_sys::console::log_1(&format!("[WASM:DEBUG] {}", msg).into());
        #[cfg(not(feature = "wasm"))]
        println!("[DEBUG] {}", msg);
    }
}

/// Initialize logger from JavaScript
#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = initLogger)]
pub fn init_logger(verbose: bool) {
    set_verbose(verbose);
}
