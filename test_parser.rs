use std::fs;
use std::path::Path;

fn main() {
    let wasmbench_dir = "/data/.openclaw/workspace/wasm-r3/benchmarks/wasm-reduce-bench/programs";
    
    let entries = fs::read_dir(wasmbench_dir).unwrap();
    let mut passed = 0;
    let mut failed = 0;
    let mut errors: Vec<(String, String)> = Vec::new();
    
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            print!("Testing {}... ", filename);
            
            match fs::read(&path) {
                Ok(bytes) => {
                    // Try to parse using wasm_parser
                    let result = std::panic::catch_unwind(|| {
                        // This would call the actual parser
                        // For now, just check if it's valid WASM magic
                        if bytes.len() >= 8 && &bytes[0..4] == &[0x00, 0x61, 0x73, 0x6d] {
                            Ok(())
                        } else {
                            Err("Invalid magic".to_string())
                        }
                    });
                    
                    match result {
                        Ok(Ok(())) => {
                            println!("OK");
                            passed += 1;
                        }
                        Ok(Err(e)) => {
                            println!("FAILED: {}", e);
                            failed += 1;
                            errors.push((filename, e));
                        }
                        Err(_) => {
                            println!("PANIC");
                            failed += 1;
                            errors.push((filename, "Panic".to_string()));
                        }
                    }
                }
                Err(e) => {
                    println!("READ ERROR: {}", e);
                    failed += 1;
                }
            }
        }
    }
    
    println!("\n=== Summary ===");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    
    if !errors.is_empty() {
        println!("\n=== Errors ===");
        for (file, error) in errors {
            println!("{}: {}", file, error);
        }
    }
}
