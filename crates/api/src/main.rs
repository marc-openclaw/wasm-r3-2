use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Serialize)]
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

#[derive(Serialize)]
struct ImportInfo {
    module: String,
    name: String,
    kind: String,
}

#[derive(Serialize)]
struct ExportInfo {
    name: String,
    kind: String,
    index: u32,
}

#[derive(Serialize)]
struct FunctionInfo {
    index: usize,
    type_index: u32,
    params: Vec<String>,
    results: Vec<String>,
    locals: usize,
    instructions: usize,
}

#[derive(Serialize)]
struct ParsedModule {
    info: ModuleInfo,
    imports: Vec<ImportInfo>,
    exports: Vec<ExportInfo>,
    functions: Vec<FunctionInfo>,
}

#[derive(Deserialize)]
struct ParseRequest {
    wasm: String, // base64 encoded
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/parse", post(parse_wasm))
        .route("/validate", post(validate_wasm))
        .layer(cors)
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)); // 10MB limit

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 wasm-api running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Json<ApiResponse<String>> {
    Json(ApiResponse {
        success: true,
        data: Some("wasm-parser API".to_string()),
        error: None,
    })
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn parse_wasm(body: axum::extract::Json<ParseRequest>) -> Result<Json<ApiResponse<ParsedModule>>, StatusCode> {
    use base64::Engine;
    let bytes = match base64::engine::general_purpose::STANDARD.decode(&body.wasm) {
        Ok(b) => b,
        Err(e) => {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Base64 decode error: {}", e)),
            }));
        }
    };

    match wasm_parser::parse(&bytes) {
        Ok(module) => {
            let info = ModuleInfo {
                functions: module.funcs.len(),
                types: module.types.len(),
                imports: module.imports.len(),
                exports: module.exports.len(),
                memories: module.memories.len(),
                tables: module.tables.len(),
                globals: module.globals.len(),
                data_segments: module.data.len(),
                element_segments: module.elements.len(),
            };

            let imports: Vec<ImportInfo> = module
                .imports
                .iter()
                .map(|i| ImportInfo {
                    module: i.module.clone(),
                    name: i.name.clone(),
                    kind: format!("{:?}", i.kind),
                })
                .collect();

            let exports: Vec<ExportInfo> = module
                .exports
                .iter()
                .map(|e| ExportInfo {
                    name: e.name.clone(),
                    kind: format!("{:?}", e.kind),
                    index: e.idx,
                })
                .collect();

            let import_count = module
                .imports
                .iter()
                .filter(|i| matches!(i.kind, wasm_parser::types::ExternalKind::Func))
                .count();

            let functions: Vec<FunctionInfo> = module
                .funcs
                .iter()
                .enumerate()
                .map(|(idx, type_idx)| {
                    let type_idx_val = *type_idx as usize;
                    let (params, results) = module
                        .types
                        .get(type_idx_val)
                        .map(|t| {
                            let p = t.params.iter().map(|pt| format!("{:?}", pt)).collect();
                            let r = t.results.iter().map(|rt| format!("{:?}", rt)).collect();
                            (p, r)
                        })
                        .unwrap_or_default();

                    let code_idx = idx.saturating_sub(import_count);
                    let (locals, instructions) = module
                        .code
                        .get(code_idx)
                        .map(|c| (c.locals.len(), c.instructions.len()))
                        .unwrap_or((0, 0));

                    FunctionInfo {
                        index: idx,
                        type_index: *type_idx,
                        params,
                        results,
                        locals,
                        instructions,
                    }
                })
                .collect();

            Ok(Json(ApiResponse {
                success: true,
                data: Some(ParsedModule {
                    info,
                    imports,
                    exports,
                    functions,
                }),
                error: None,
            }))
        }
        Err(e) => Ok(Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        })),
    }
}

async fn validate_wasm(body: axum::extract::Json<ParseRequest>) -> Result<Json<ApiResponse<bool>>, StatusCode> {
    use base64::Engine;
    let bytes = match base64::engine::general_purpose::STANDARD.decode(&body.wasm) {
        Ok(b) => b,
        Err(e) => {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Base64 decode error: {}", e)),
            }));
        }
    };

    match wasm_parser::parse(&bytes) {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            data: Some(true),
            error: None,
        })),
        Err(e) => Ok(Json(ApiResponse {
            success: false,
            data: Some(false),
            error: Some(e.to_string()),
        })),
    }
}
