use std::fs;
use wasm_parser::decode::parse_bytes;

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
            
            match fs::read(&path) {
                Ok(bytes) => {
                    match parse_bytes(&bytes) {
                        Ok(module) => {
                            println!("{}: OK ({} types, {} funcs, {} exports)", 
                                filename, 
                                module.types.len(),
                                module.funcs.len(),
                                module.exports.len()
                            );
                            passed += 1;
                        }
                        Err(e) => {
                            println!("{}: FAILED - {:?}", filename, e);
                            failed += 1;
                            errors.push((filename, format!("{:?}", e)));
                        }
                    }
                }
                Err(e) => {
                    println!("{}: READ ERROR - {}", filename, e);
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
        for (file, error) in &errors {
            println!("{}: {}", file, error);
        }
    }
    
    if failed > 0 {
        std::process::exit(1);
    }
}
