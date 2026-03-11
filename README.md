# wasm-r3-2

[![Crates.io](https://img.shields.io/crates/v/wasm_parser)](https://crates.io/crates/wasm_parser)
[![Docs.rs](https://docs.rs/wasm_parser/badge.svg)](https://docs.rs/wasm_parser)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust crate for parsing, modifying, and encoding WebAssembly binary modules with an editable AST structure.

## Features

- **Parse** WebAssembly binaries into an editable AST
- **Modify** modules programmatically (add/remove functions, instructions, etc.)
- **Encode** modified modules back to WASM binary format
- **WASM Support** - Compile this crate to WebAssembly for use in browsers
- **Zero-copy** parsing where possible
- Full support for WebAssembly MVP and most proposals

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
wasm_parser = "0.1.0"
```

For WASM support:

```toml
[dependencies]
wasm_parser = { version = "0.1.0", features = ["wasm"] }
```

## Building

### Native Build

```bash
cargo build --release
```

### WASM Build

Build for WebAssembly target:

```bash
# Install wasm-pack if you haven't already
cargo install wasm-pack

# Build for browser
wasm-pack build --target web --features wasm

# Build for Node.js
wasm-pack build --target nodejs --features wasm
```

## Testing

```bash
cargo test
```

## API Usage

### Basic Parsing

Parse a WASM binary file into an editable AST:

```rust
use wasm_parser::parser::WasmParser;
use wasm_parser::{parse, encode_module};

// Parse from bytes
let wasm_bytes = std::fs::read("module.wasm")?;
let module = parse(&wasm_bytes)?;

// Or use the high-level API
let module = WasmParser::parse(&wasm_bytes)?;
```

### Inspecting Modules

```rust
use wasm_parser::ModuleExt;

// Get function count
let func_count = module.function_count();

// Check if a function exists
if module.has_function(0) {
    println!("Function 0 exists");
}

// Get function body
if let Some(body) = module.get_function_body(0) {
    println!("Function has {} instructions", body.instructions.len());
}
```

### Modifying Modules

```rust
use wasm_parser::{Module, FunctionBody, Instruction};
use wasm_parser::types::{FuncType, ValueType};

let mut module = Module::new();

// Add a function type
let type_idx = module.add_type(FuncType::new(
    vec![ValueType::I32, ValueType::I32],  // params
    vec![ValueType::I32],                   // results
));

// Add a function
let func_idx = module.add_function(type_idx);

// Create function body
let mut body = FunctionBody::new();
body.add_local(1, ValueType::I32);  // Add local variable

// Add instructions
body.add_instruction(Instruction::LocalGet { local_idx: 0 });
body.add_instruction(Instruction::LocalGet { local_idx: 1 });
body.add_instruction(Instruction::I32Add);
body.add_instruction(Instruction::End);

// Add function body to module
module.code.push(body);
```

### Modifying Existing Functions

```rust
use wasm_parser::ModuleExt;
use wasm_parser::instruction::Instruction;

// Modify a function's instructions
module.modify_function(0, |body| {
    // Add instruction at end
    body.add_instruction(Instruction::Nop);
    
    // Insert instruction at position
    body.insert_instruction(0, Instruction::Block { block_type: BlockType::Empty });
    
    // Remove instruction
    body.remove_instruction(5);
    
    // Replace instruction
    body.replace_instruction(2, Instruction::I32Const { value: 42 });
});

// Or use individual methods
module.insert_instruction(0, 0, Instruction::Nop)?;
module.replace_instruction(0, 1, Instruction::Unreachable)?;
let removed = module.remove_instruction(0, 2)?;
```

### Adding Imports and Exports

```rust
use wasm_parser::ast::{Import, Export};
use wasm_parser::types::ExternalKind;

// Add an import
module.add_import(Import {
    module: "env".to_string(),
    name: "memory".to_string(),
    kind: ExternalKind::Mem,
    idx: 0,
});

// Add an export
module.add_export("add".to_string(), ExternalKind::Func, 0);
```

### Working with Memory and Tables

```rust
use wasm_parser::types::{MemType, TableType, Limits, ValueType};

// Add memory
module.add_memory(MemType {
    limits: Limits::new(1, Some(10)),  // min: 1 page, max: 10 pages
});

// Add table
module.add_table(TableType {
    limits: Limits::new(10, Some(100)),
    elem_type: ValueType::FuncRef,
});
```

### Encoding Back to WASM

```rust
use wasm_parser::encode_module;

// Encode module to bytes
let output_bytes = encode_module(&module)?;

// Write to file
std::fs::write("output.wasm", output_bytes)?;

// Or use the high-level API
WasmParser::write_file(std::path::Path::new("output.wasm"), &module)?;
```

### Full Example: Add Logging to All Functions

```rust
use wasm_parser::{parse, encode_module};
use wasm_parser::instruction::Instruction;
use wasm_parser::types::ExternalKind;

fn main() -> anyhow::Result<()> {
    // Parse WASM module
    let bytes = std::fs::read("input.wasm")?;
    let mut module = parse(&bytes)?;
    
    // Add import for console.log (assuming env.print exists)
    let import_idx = module.add_import(Import {
        module: "env".to_string(),
        name: "print".to_string(),
        kind: ExternalKind::Func,
        idx: 0, // type index
    });
    
    // For each function, add call to print at start
    for func_idx in 0..module.function_count() as u32 {
        module.insert_instruction(func_idx, 0, Instruction::I32Const { value: func_idx as i32 })?;
        module.insert_instruction(func_idx, 1, Instruction::Call { function_index: import_idx })?;
    }
    
    // Encode and save
    let output = encode_module(&module)?;
    std::fs::write("output.wasm", output)?;
    
    Ok(())
}
```

## WASM JavaScript API

When compiled to WASM, the crate provides a JavaScript-friendly API:

```javascript
import init, { WasmModule } from './pkg/wasm_parser.js';

async function run() {
    await init();
    
    // Parse WASM from bytes
    const response = await fetch('module.wasm');
    const bytes = new Uint8Array(await response.arrayBuffer());
    const module = WasmModule.parse(bytes);
    
    // Inspect module
    console.log('Functions:', module.functionCount);
    console.log('Types:', module.typeCount);
    console.log('Exports:', module.exportCount);
    console.log('Imports:', module.importCount);
    console.log('Memory:', module.memoryCount);
    console.log('Tables:', module.tableCount);
    console.log('Globals:', module.globalCount);
    console.log('Data:', module.dataCount);
    console.log('Elements:', module.elementCount);
    
    // Get module as JSON
    const json = module.toJSON();
    console.log('Module structure:', JSON.parse(json));
    
    // Encode back to bytes
    const output = module.encode();
}

run();
```

## Module Structure

The `Module` struct provides access to all WASM sections:

| Field | Description |
|-------|-------------|
| `custom_sections` | Custom sections (debug info, names, etc.) |
| `types` | Function type definitions |
| `imports` | Imported functions, memories, tables, globals |
| `funcs` | Function type indices |
| `tables` | Table definitions |
| `memories` | Memory definitions |
| `globals` | Global variable definitions |
| `exports` | Exported items |
| `start` | Start function index |
| `elements` | Element segments (function tables) |
| `code` | Function bodies |
| `data` | Data segments |
| `data_count` | Data count (for bulk memory) |

## Crate Features

| Feature | Description |
|---------|-------------|
| `default` | Standard library support |
| `wasm` | WASM bindings for browser/JS use |

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions welcome! Please open an issue or pull request on GitHub.

## Related Projects

- [wasmparser](https://github.com/bytecodealliance/wasm-tools) - Low-level streaming parser
- [wasmtime](https://github.com/bytecodealliance/wasmtime) - WASM runtime
- [wasm-pack](https://github.com/rustwasm/wasm-pack) - Build tool for Rust-generated WASM
