/* tslint:disable */
/* eslint-disable */

/**
 * WASM module wrapper for JavaScript interop
 */
export class WasmModule {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Encode module to bytes as a Uint8Array
     */
    encode(): Uint8Array;
    /**
     * Create a new empty module
     */
    constructor();
    /**
     * Parse WASM binary from a JavaScript Uint8Array
     */
    static parse(data: Uint8Array): WasmModule;
    /**
     * Get module as JSON representation
     */
    toJSON(): string;
    /**
     * Get data segment count
     */
    readonly dataCount: number;
    /**
     * Get element segment count
     */
    readonly elementCount: number;
    /**
     * Get export count
     */
    readonly exportCount: number;
    /**
     * Get function count
     */
    readonly functionCount: number;
    /**
     * Get global count
     */
    readonly globalCount: number;
    /**
     * Get import count
     */
    readonly importCount: number;
    /**
     * Get memory count
     */
    readonly memoryCount: number;
    /**
     * Get table count
     */
    readonly tableCount: number;
    /**
     * Get type count
     */
    readonly typeCount: number;
}

/**
 * Initialize WASM module (call once from JS)
 */
export function start(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmmodule_free: (a: number, b: number) => void;
    readonly start: () => void;
    readonly wasmmodule_dataCount: (a: number) => number;
    readonly wasmmodule_elementCount: (a: number) => number;
    readonly wasmmodule_encode: (a: number) => [number, number, number];
    readonly wasmmodule_exportCount: (a: number) => number;
    readonly wasmmodule_functionCount: (a: number) => number;
    readonly wasmmodule_globalCount: (a: number) => number;
    readonly wasmmodule_importCount: (a: number) => number;
    readonly wasmmodule_memoryCount: (a: number) => number;
    readonly wasmmodule_new: () => number;
    readonly wasmmodule_parse: (a: any) => [number, number, number];
    readonly wasmmodule_tableCount: (a: number) => number;
    readonly wasmmodule_toJSON: (a: number) => [number, number, number, number];
    readonly wasmmodule_typeCount: (a: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
