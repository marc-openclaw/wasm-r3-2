use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <wasm-file>", args[0]);
        return;
    }
    
    let path = &args[1];
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to read {}: {}", path, e);
            return;
        }
    };
    
    println!("Testing {} ({} bytes)...", path, bytes.len());
    
    match wasm_parser::parse(&bytes) {
        Ok(module) => {
            println!("✓ Parsed successfully");
            println!("  Functions: {}", module.funcs.len());
            println!("  Types: {}", module.types.len());
            println!("  Imports: {}", module.imports.len());
            println!("  Exports: {}", module.exports.len());
        }
        Err(e) => {
            eprintln!("✗ Parse failed: {}", e);
        }
    }
}