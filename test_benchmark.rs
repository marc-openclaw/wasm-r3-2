use std::fs;
use std::path::Path;

fn main() {
    let benchmark_dir = "/data/.openclaw/workspace/wasm-r3/benchmarks/wasm-r3-bench";
    
    // Get list of wasm files
    let entries = match fs::read_dir(benchmark_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to read benchmark directory: {}", e);
            return;
        }
    };
    
    let mut wasm_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .map(|ext| ext == "wasm")
                .unwrap_or(false)
        })
        .map(|e| e.path())
        .collect();
    
    wasm_files.sort();
    
    println!("Testing wasm-r3-2 parser on {} WASM files from wasm-r3-bench", wasm_files.len());
    println!("{}", "=".repeat(80));
    
    let mut passed = 0;
    let mut failed = 0;
    
    for wasm_file in &wasm_files {
        let file_name = wasm_file.file_name().unwrap().to_str().unwrap();
        let bytes = match fs::read(wasm_file) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Failed to read {}: {}", file_name, e);
                failed += 1;
                continue;
            }
        };
        
        // Try to parse the WASM file
        match wasm_parser::parse(&bytes) {
            Ok(module) => {
                println!("✓ {} - Parsed successfully", file_name);
                println!("  Functions: {}, Types: {}, Imports: {}, Exports: {}",
                    module.funcs.len(),
                    module.types.len(),
                    module.imports.len(),
                    module.exports.len()
                );
                passed += 1;
            }
            Err(e) => {
                eprintln!("✗ {} - Parse failed: {}", file_name, e);
                failed += 1;
            }
        }
    }
    
    println!("{}", "=".repeat(80));
    println!("Results: {} passed, {} failed out of {} files", passed, failed, wasm_files.len());
    
    if failed > 0 {
        std::process::exit(1);
    }
}
