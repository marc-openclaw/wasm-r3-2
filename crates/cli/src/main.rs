use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wasm_parser::{parse, encode_module};

#[derive(Parser)]
#[command(name = "wasm-cli")]
#[command(about = "CLI tool for WebAssembly module analysis")]
#[command(version)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a WASM file and display info
    Parse {
        /// Path to WASM file
        file: PathBuf,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show detailed module statistics
    Stats {
        /// Path to WASM file
        file: PathBuf,
    },
    /// Extract imports from WASM module
    Imports {
        /// Path to WASM file
        file: PathBuf,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Extract exports from WASM module
    Exports {
        /// Path to WASM file
        file: PathBuf,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show function signatures
    Functions {
        /// Path to WASM file
        file: PathBuf,
        /// Show function bodies
        #[arg(long)]
        bodies: bool,
    },
    /// Validate a WASM file
    Validate {
        /// Path to WASM file
        file: PathBuf,
    },
    /// Encode a module back to WASM (roundtrip test)
    Roundtrip {
        /// Input WASM file
        input: PathBuf,
        /// Output WASM file
        output: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { file, json } => {
            let bytes = std::fs::read(&file)?;
            let module = parse(&bytes)?;

            if json {
                let info = ModuleInfo::from(&module);
                println!("{}", serde_json::to_string_pretty(&info)?);
            } else {
                println!("File: {}", file.display());
                println!("\nModule Overview:");
                println!("  Functions: {}", module.funcs.len());
                println!("  Types: {}", module.types.len());
                println!("  Imports: {}", module.imports.len());
                println!("  Exports: {}", module.exports.len());
                println!("  Memories: {}", module.memories.len());
                println!("  Tables: {}", module.tables.len());
                println!("  Globals: {}", module.globals.len());
                println!("  Data segments: {}", module.data.len());
                println!("  Element segments: {}", module.elements.len());

                if !module.imports.is_empty() {
                    println!("\nImports:");
                    for imp in &module.imports {
                        println!("  {}.{} ({:?})", imp.module, imp.name, imp.kind);
                    }
                }

                if !module.exports.is_empty() {
                    println!("\nExports:");
                    for exp in &module.exports {
                        println!("  {} ({:?}) index {}", exp.name, exp.kind, exp.idx);
                    }
                }
            }
        }
        Commands::Stats { file } => {
            let bytes = std::fs::read(&file)?;
            let module = parse(&bytes)?;

            println!("File: {}", file.display());
            println!("File size: {} bytes", bytes.len());
            println!("\nModule Statistics:");
            println!("  Functions: {}", module.funcs.len());
            println!("  Types: {}", module.types.len());
            println!("  Imports: {}", module.imports.len());
            println!("  Exports: {}", module.exports.len());
            println!("  Memories: {}", module.memories.len());
            println!("  Tables: {}", module.tables.len());
            println!("  Globals: {}", module.globals.len());
            println!("  Data segments: {}", module.data.len());
            println!("  Element segments: {}", module.elements.len());

            let code_size: usize = module.code.iter().map(|c| {
                c.instructions.len() * std::mem::size_of::<wasm_parser::instruction::Instruction>()
            }).sum();
            println!("\nEstimated code size: {} bytes", code_size);

            let data_size: usize = module.data.iter().map(|d| d.data.len()).sum();
            println!("Total data segment size: {} bytes", data_size);
        }
        Commands::Imports { file, json } => {
            let bytes = std::fs::read(&file)?;
            let module = parse(&bytes)?;

            if json {
                let imports: Vec<ImportInfo> = module.imports.iter().map(|i| ImportInfo {
                    module: i.module.clone(),
                    name: i.name.clone(),
                    kind: format!("{:?}", i.kind),
                }).collect();
                println!("{}", serde_json::to_string_pretty(&imports)?);
            } else {
                println!("Imports from {}:", file.display());
                for imp in &module.imports {
                    println!("  {}.{} ({:?})", imp.module, imp.name, imp.kind);
                }
            }
        }
        Commands::Exports { file, json } => {
            let bytes = std::fs::read(&file)?;
            let module = parse(&bytes)?;

            if json {
                let exports: Vec<ExportInfo> = module.exports.iter().map(|e| ExportInfo {
                    name: e.name.clone(),
                    kind: format!("{:?}", e.kind),
                    index: e.idx,
                }).collect();
                println!("{}", serde_json::to_string_pretty(&exports)?);
            } else {
                println!("Exports from {}:", file.display());
                for exp in &module.exports {
                    println!("  {} ({:?}) index {}", exp.name, exp.kind, exp.idx);
                }
            }
        }
        Commands::Functions { file, bodies } => {
            let bytes = std::fs::read(&file)?;
            let module = parse(&bytes)?;

            println!("Functions in {}:", file.display());
            for (idx, func_type_idx) in module.funcs.iter().enumerate() {
                let type_idx = *func_type_idx as usize;
                if let Some(func_type) = module.types.get(type_idx) {
                    let params: Vec<String> = func_type.params.iter().map(|t| format!("{:?}", t)).collect();
                    let results: Vec<String> = func_type.results.iter().map(|t| format!("{:?}", t)).collect();
                    println!("  Function {}: type {} -> params [{}], results [{}]",
                        idx, type_idx, params.join(", "), results.join(", "));
                }

                if bodies {
                    let import_count = module.imports.iter()
                        .filter(|i| matches!(i.kind, wasm_parser::types::ExternalKind::Func))
                        .count();
                    let code_idx = idx.saturating_sub(import_count);
                    if let Some(body) = module.code.get(code_idx) {
                        println!("    Locals: {}", body.locals.len());
                        println!("    Instructions: {}", body.instructions.len());
                    }
                }
            }
        }
        Commands::Validate { file } => {
            match std::fs::read(&file) {
                Ok(bytes) => {
                    match parse(&bytes) {
                        Ok(_) => {
                            println!("✓ {} is valid WebAssembly", file.display());
                        }
                        Err(e) => {
                            println!("✗ {} failed validation: {}", file.display(), e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Error reading {}: {}", file.display(), e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Roundtrip { input, output } => {
            let bytes = std::fs::read(&input)?;
            let module = parse(&bytes)?;
            let encoded = encode_module(&module)?;
            std::fs::write(&output, encoded)?;
            println!("Roundtrip: {} -> {}", input.display(), output.display());
        }
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct ModuleInfo {
    functions: usize,
    types: usize,
    imports: usize,
    exports: usize,
    memories: usize,
    tables: usize,
    globals: usize,
    data_segments: usize,
    element_segments: usize,
}

impl From<&wasm_parser::ast::Module> for ModuleInfo {
    fn from(m: &wasm_parser::ast::Module) -> Self {
        Self {
            functions: m.funcs.len(),
            types: m.types.len(),
            imports: m.imports.len(),
            exports: m.exports.len(),
            memories: m.memories.len(),
            tables: m.tables.len(),
            globals: m.globals.len(),
            data_segments: m.data.len(),
            element_segments: m.elements.len(),
        }
    }
}

#[derive(serde::Serialize)]
struct ImportInfo {
    module: String,
    name: String,
    kind: String,
}

#[derive(serde::Serialize)]
struct ExportInfo {
    name: String,
    kind: String,
    index: u32,
}
