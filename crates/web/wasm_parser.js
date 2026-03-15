/* @ts-self-types="./wasm_parser.d.ts" */

/**
 * WASM module wrapper for JavaScript interop
 */
export class WasmModule {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmModule.prototype);
        obj.__wbg_ptr = ptr;
        WasmModuleFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmModuleFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmmodule_free(ptr, 0);
    }
    /**
     * Get data segment count
     * @returns {number}
     */
    get dataCount() {
        const ret = wasm.wasmmodule_dataCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get element segment count
     * @returns {number}
     */
    get elementCount() {
        const ret = wasm.wasmmodule_elementCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Encode module to bytes as a Uint8Array
     * @returns {Uint8Array}
     */
    encode() {
        const ret = wasm.wasmmodule_encode(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Get export count
     * @returns {number}
     */
    get exportCount() {
        const ret = wasm.wasmmodule_exportCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get function count
     * @returns {number}
     */
    get functionCount() {
        const ret = wasm.wasmmodule_functionCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get global count
     * @returns {number}
     */
    get globalCount() {
        const ret = wasm.wasmmodule_globalCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get import count
     * @returns {number}
     */
    get importCount() {
        const ret = wasm.wasmmodule_importCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get memory count
     * @returns {number}
     */
    get memoryCount() {
        const ret = wasm.wasmmodule_memoryCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Create a new empty module
     */
    constructor() {
        const ret = wasm.wasmmodule_new();
        this.__wbg_ptr = ret >>> 0;
        WasmModuleFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Parse WASM binary from a JavaScript Uint8Array
     * @param {Uint8Array} data
     * @returns {WasmModule}
     */
    static parse(data) {
        const ret = wasm.wasmmodule_parse(data);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmModule.__wrap(ret[0]);
    }
    /**
     * Get table count
     * @returns {number}
     */
    get tableCount() {
        const ret = wasm.wasmmodule_tableCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get module as JSON representation
     * @returns {string}
     */
    toJSON() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.wasmmodule_toJSON(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get type count
     * @returns {number}
     */
    get typeCount() {
        const ret = wasm.wasmmodule_typeCount(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) WasmModule.prototype[Symbol.dispose] = WasmModule.prototype.free;

/**
 * Initialize logger from JavaScript
 * @param {boolean} verbose
 */
export function initLogger(verbose) {
    wasm.initLogger(verbose);
}

/**
 * Initialize WASM module (call once from JS)
 */
export function start() {
    wasm.start();
}

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_6ddd609b62940d55: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_error_8d9a8e04cd1d3588: function(arg0) {
            console.error(arg0);
        },
        __wbg_length_ea16607d7b61445b: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_log_524eedafa26daa59: function(arg0) {
            console.log(arg0);
        },
        __wbg_new_d15cb560a6a0e5f0: function(arg0, arg1) {
            const ret = new Error(getStringFromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_new_with_length_825018a1616e9e55: function(arg0) {
            const ret = new Uint8Array(arg0 >>> 0);
            return ret;
        },
        __wbg_prototypesetcall_d62e5099504357e6: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
        },
        __wbg_set_8c0b3ffcf05d61c2: function(arg0, arg1, arg2) {
            arg0.set(getArrayU8FromWasm0(arg1, arg2));
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./wasm_parser_bg.js": import0,
    };
}

const WasmModuleFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmmodule_free(ptr >>> 0, 1));

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('wasm_parser_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
