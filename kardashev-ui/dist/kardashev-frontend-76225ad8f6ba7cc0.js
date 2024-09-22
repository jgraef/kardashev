let wasm;

const heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

let cachedUint8Memory0 = null;

function getUint8Memory0() {
    if (cachedUint8Memory0 === null || cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachedInt32Memory0 = null;

function getInt32Memory0() {
    if (cachedInt32Memory0 === null || cachedInt32Memory0.byteLength === 0) {
        cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32Memory0;
}

let cachedFloat64Memory0 = null;

function getFloat64Memory0() {
    if (cachedFloat64Memory0 === null || cachedFloat64Memory0.byteLength === 0) {
        cachedFloat64Memory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64Memory0;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => {
    wasm.__wbindgen_export_2.get(state.dtor)(state.a, state.b)
});

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}
function __wbg_adapter_40(arg0, arg1, arg2, arg3) {
    wasm._dyn_core__ops__function__FnMut__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h617d76d3c7a6d40f(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

function __wbg_adapter_43(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hc05ee6fcd19601c1(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_46(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__he3a51359577be599(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_49(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h15377132cc55bf6a(arg0, arg1);
}

function __wbg_adapter_52(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h6060c9f653c409b7(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_55(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hf62e706c5c6e48d4(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_58(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hc9f1411e601b02df(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_61(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h02b72553b27f7a5c(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_64(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h3cde4153f0b4262a(arg0, arg1);
}

function __wbg_adapter_67(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h6903044e1e87e0e1(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_70(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__ha11b13032f1c4d25(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_73(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hae80802a448e4ed2(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_76(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h2a53bbf487aeb6cb(arg0, arg1);
}

function __wbg_adapter_79(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hbaece0402db152eb(arg0, arg1, addHeapObject(arg2));
}

function getCachedStringFromWasm0(ptr, len) {
    if (ptr === 0) {
        return getObject(len);
    } else {
        return getStringFromWasm0(ptr, len);
    }
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}

let cachedUint32Memory0 = null;

function getUint32Memory0() {
    if (cachedUint32Memory0 === null || cachedUint32Memory0.byteLength === 0) {
        cachedUint32Memory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32Memory0;
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32Memory0().subarray(ptr / 4, ptr / 4 + len);
}
function __wbg_adapter_756(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures__invoke2_mut__h6999590d03de3d7f(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

const IntoUnderlyingByteSourceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingbytesource_free(ptr >>> 0));
/**
*/
export class IntoUnderlyingByteSource {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IntoUnderlyingByteSourceFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingbytesource_free(ptr);
    }
    /**
    * @returns {string}
    */
    get type() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.intounderlyingbytesource_type(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v1 = getCachedStringFromWasm0(r0, r1);
        if (r0 !== 0) { wasm.__wbindgen_free(r0, r1, 1); }
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
/**
* @returns {number}
*/
get autoAllocateChunkSize() {
    const ret = wasm.intounderlyingbytesource_autoAllocateChunkSize(this.__wbg_ptr);
    return ret >>> 0;
}
/**
* @param {ReadableByteStreamController} controller
*/
start(controller) {
    wasm.intounderlyingbytesource_start(this.__wbg_ptr, addHeapObject(controller));
}
/**
* @param {ReadableByteStreamController} controller
* @returns {Promise<any>}
*/
pull(controller) {
    const ret = wasm.intounderlyingbytesource_pull(this.__wbg_ptr, addHeapObject(controller));
    return takeObject(ret);
}
/**
*/
cancel() {
    const ptr = this.__destroy_into_raw();
    wasm.intounderlyingbytesource_cancel(ptr);
}
}

const IntoUnderlyingSinkFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsink_free(ptr >>> 0));
/**
*/
export class IntoUnderlyingSink {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IntoUnderlyingSinkFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingsink_free(ptr);
    }
    /**
    * @param {any} chunk
    * @returns {Promise<any>}
    */
    write(chunk) {
        const ret = wasm.intounderlyingsink_write(this.__wbg_ptr, addHeapObject(chunk));
        return takeObject(ret);
    }
    /**
    * @returns {Promise<any>}
    */
    close() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.intounderlyingsink_close(ptr);
        return takeObject(ret);
    }
    /**
    * @param {any} reason
    * @returns {Promise<any>}
    */
    abort(reason) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.intounderlyingsink_abort(ptr, addHeapObject(reason));
        return takeObject(ret);
    }
}

const IntoUnderlyingSourceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsource_free(ptr >>> 0));
/**
*/
export class IntoUnderlyingSource {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IntoUnderlyingSourceFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingsource_free(ptr);
    }
    /**
    * @param {ReadableStreamDefaultController} controller
    * @returns {Promise<any>}
    */
    pull(controller) {
        const ret = wasm.intounderlyingsource_pull(this.__wbg_ptr, addHeapObject(controller));
        return takeObject(ret);
    }
    /**
    */
    cancel() {
        const ptr = this.__destroy_into_raw();
        wasm.intounderlyingsource_cancel(ptr);
    }
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
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
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        const ret = getObject(arg0) === undefined;
        return ret;
    };
    imports.wbg.__wbindgen_in = function(arg0, arg1) {
        const ret = getObject(arg0) in getObject(arg1);
        return ret;
    };
    imports.wbg.__wbindgen_is_null = function(arg0) {
        const ret = getObject(arg0) === null;
        return ret;
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_is_array = function(arg0) {
        const ret = Array.isArray(getObject(arg0));
        return ret;
    };
    imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
        const ret = getObject(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_error_f851667af71bcfc6 = function(arg0, arg1) {
        var v0 = getCachedStringFromWasm0(arg0, arg1);
    if (arg0 !== 0) { wasm.__wbindgen_free(arg0, arg1, 1); }
    console.error(v0);
};
imports.wbg.__wbg_new_abda76e883ba8a5f = function() {
    const ret = new Error();
    return addHeapObject(ret);
};
imports.wbg.__wbg_stack_658279fe44541cf6 = function(arg0, arg1) {
    const ret = getObject(arg1).stack;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_mark_40e050a77cc39fea = function(arg0, arg1) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
    performance.mark(v0);
};
imports.wbg.__wbg_measure_aa7a73f17813f708 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
if (arg0 !== 0) { wasm.__wbindgen_free(arg0, arg1, 1); }
var v1 = getCachedStringFromWasm0(arg2, arg3);
if (arg2 !== 0) { wasm.__wbindgen_free(arg2, arg3, 1); }
performance.measure(v0, v1);
}, arguments) };
imports.wbg.__wbg_log_c9486ca5d8e2cbe8 = function(arg0, arg1) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
if (arg0 !== 0) { wasm.__wbindgen_free(arg0, arg1, 1); }
console.log(v0);
};
imports.wbg.__wbg_log_aba5996d9bde071f = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
if (arg0 !== 0) { wasm.__wbindgen_free(arg0, arg1, 1); }
var v1 = getCachedStringFromWasm0(arg2, arg3);
var v2 = getCachedStringFromWasm0(arg4, arg5);
var v3 = getCachedStringFromWasm0(arg6, arg7);
console.log(v0, v1, v2, v3);
};
imports.wbg.__wbindgen_cb_drop = function(arg0) {
    const obj = takeObject(arg0).original;
    if (obj.cnt-- == 1) {
        obj.a = 0;
        return true;
    }
    const ret = false;
    return ret;
};
imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_Window_cc0273a5da2c36dc = function(arg0) {
    const ret = getObject(arg0).Window;
    return addHeapObject(ret);
};
imports.wbg.__wbg_scheduler_6932606c19435996 = function(arg0) {
    const ret = getObject(arg0).scheduler;
    return addHeapObject(ret);
};
imports.wbg.__wbg_requestIdleCallback_081ddac93612a53e = function(arg0) {
    const ret = getObject(arg0).requestIdleCallback;
    return addHeapObject(ret);
};
imports.wbg.__wbg_scheduler_8082c844a9cfc0df = function(arg0) {
    const ret = getObject(arg0).scheduler;
    return addHeapObject(ret);
};
imports.wbg.__wbg_postTask_4674878f9a603824 = function(arg0, arg1, arg2) {
    const ret = getObject(arg0).postTask(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
};
imports.wbg.__wbg_offsetX_d08eda91526f22a2 = function(arg0) {
    const ret = getObject(arg0).offsetX;
    return ret;
};
imports.wbg.__wbg_offsetY_3c895bb1534dfbf4 = function(arg0) {
    const ret = getObject(arg0).offsetY;
    return ret;
};
imports.wbg.__wbg_onpointerrawupdate_e087759b4021ec00 = function(arg0) {
    const ret = getObject(arg0).onpointerrawupdate;
    return addHeapObject(ret);
};
imports.wbg.__wbg_getCoalescedEvents_4665669d237be577 = function(arg0) {
    const ret = getObject(arg0).getCoalescedEvents;
    return addHeapObject(ret);
};
imports.wbg.__wbg_requestFullscreen_f4349fb8a7429cf9 = function(arg0) {
    const ret = getObject(arg0).requestFullscreen();
    return addHeapObject(ret);
};
imports.wbg.__wbg_webkitRequestFullscreen_8abcfecec7127495 = function(arg0) {
    getObject(arg0).webkitRequestFullscreen();
};
imports.wbg.__wbg_webkitFullscreenElement_533c5f32e2ac8d0c = function(arg0) {
    const ret = getObject(arg0).webkitFullscreenElement;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_webkitExitFullscreen_225988f6e8c97b63 = function(arg0) {
    getObject(arg0).webkitExitFullscreen();
};
imports.wbg.__wbg_requestFullscreen_a851d70cb190396a = function(arg0) {
    const ret = getObject(arg0).requestFullscreen;
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_number_new = function(arg0) {
    const ret = arg0;
    return addHeapObject(ret);
};
imports.wbg.__wbg_prototype_8e5075a5dd95f801 = function() {
    const ret = ResizeObserverEntry.prototype;
    return addHeapObject(ret);
};
imports.wbg.__wbg_performance_eeefc685c9bc38b4 = function(arg0) {
    const ret = getObject(arg0).performance;
    return addHeapObject(ret);
};
imports.wbg.__wbg_now_e0d8ec93dd25766a = function(arg0) {
    const ret = getObject(arg0).now();
    return ret;
};
imports.wbg.__wbg_Window_a1459b9c171b6eed = function(arg0) {
    const ret = getObject(arg0).Window;
    return addHeapObject(ret);
};
imports.wbg.__wbg_WorkerGlobalScope_e1b8bcefd2818e94 = function(arg0) {
    const ret = getObject(arg0).WorkerGlobalScope;
    return addHeapObject(ret);
};
imports.wbg.__wbg_instanceof_GpuCanvasContext_05351086956f1883 = function(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUCanvasContext;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};
imports.wbg.__wbg_configure_8ae8b7e66a9d6189 = function(arg0, arg1) {
    getObject(arg0).configure(getObject(arg1));
};
imports.wbg.__wbg_getCurrentTexture_26a07297d850dcb1 = function(arg0) {
    const ret = getObject(arg0).getCurrentTexture();
    return addHeapObject(ret);
};
imports.wbg.__wbg_instanceof_GpuOutOfMemoryError_b37a08bfb7cee038 = function(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUOutOfMemoryError;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};
imports.wbg.__wbg_dispatchWorkgroups_2190ad793cd27850 = function(arg0, arg1, arg2, arg3) {
    getObject(arg0).dispatchWorkgroups(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0);
};
imports.wbg.__wbg_dispatchWorkgroupsIndirect_cfc6272439398a21 = function(arg0, arg1, arg2) {
    getObject(arg0).dispatchWorkgroupsIndirect(getObject(arg1), arg2);
};
imports.wbg.__wbg_end_a895c7d0f47bb8e0 = function(arg0) {
    getObject(arg0).end();
};
imports.wbg.__wbg_setPipeline_9730cb37968bb3d1 = function(arg0, arg1) {
    getObject(arg0).setPipeline(getObject(arg1));
};
imports.wbg.__wbg_setBindGroup_c11c5cfe30b7ec4a = function(arg0, arg1, arg2) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
};
imports.wbg.__wbg_setBindGroup_0184ac17323d75b2 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
};
imports.wbg.__wbg_instanceof_GpuDeviceLostInfo_9a6164dd8679873d = function(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUDeviceLostInfo;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};
imports.wbg.__wbg_reason_d2d2de04f9960f03 = function(arg0) {
    const ret = getObject(arg0).reason;
    return addHeapObject(ret);
};
imports.wbg.__wbg_message_b8c6a6f7bab63a0e = function(arg0, arg1) {
    const ret = getObject(arg1).message;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_end_0fafe47bdc78c53d = function(arg0) {
    getObject(arg0).end();
};
imports.wbg.__wbg_executeBundles_4bcd6c8ecfaedf51 = function(arg0, arg1) {
    getObject(arg0).executeBundles(getObject(arg1));
};
imports.wbg.__wbg_setBlendConstant_a946e294911337e9 = function(arg0, arg1) {
    getObject(arg0).setBlendConstant(getObject(arg1));
};
imports.wbg.__wbg_setScissorRect_cd8f44130fd71416 = function(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setScissorRect(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
};
imports.wbg.__wbg_setStencilReference_08db4d5601a3f285 = function(arg0, arg1) {
    getObject(arg0).setStencilReference(arg1 >>> 0);
};
imports.wbg.__wbg_setViewport_66dfe2ad99a0ccd6 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).setViewport(arg1, arg2, arg3, arg4, arg5, arg6);
};
imports.wbg.__wbg_setBindGroup_4147d4ebb7213bb3 = function(arg0, arg1, arg2) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
};
imports.wbg.__wbg_setBindGroup_96a4847ff3077350 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
};
imports.wbg.__wbg_draw_7266fe228aea02a8 = function(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
};
imports.wbg.__wbg_drawIndexed_23bcd62668716ed0 = function(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
};
imports.wbg.__wbg_drawIndexedIndirect_6f3721f18ad10b1e = function(arg0, arg1, arg2) {
    getObject(arg0).drawIndexedIndirect(getObject(arg1), arg2);
};
imports.wbg.__wbg_drawIndirect_1a15176b1b8537ff = function(arg0, arg1, arg2) {
    getObject(arg0).drawIndirect(getObject(arg1), arg2);
};
imports.wbg.__wbg_setIndexBuffer_1860608e395ec140 = function(arg0, arg1, arg2, arg3) {
    getObject(arg0).setIndexBuffer(getObject(arg1), takeObject(arg2), arg3);
};
imports.wbg.__wbg_setIndexBuffer_83f311a5a378a545 = function(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setIndexBuffer(getObject(arg1), takeObject(arg2), arg3, arg4);
};
imports.wbg.__wbg_setPipeline_d3556629635bf281 = function(arg0, arg1) {
    getObject(arg0).setPipeline(getObject(arg1));
};
imports.wbg.__wbg_setVertexBuffer_d439a224a2369412 = function(arg0, arg1, arg2, arg3) {
    getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3);
};
imports.wbg.__wbg_setVertexBuffer_0dca9fc7421bd152 = function(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3, arg4);
};
imports.wbg.__wbg_has_d655f3a252d0b10a = function(arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).has(v0);
    return ret;
};
imports.wbg.__wbg_maxTextureDimension1D_53351b4a7253c324 = function(arg0) {
    const ret = getObject(arg0).maxTextureDimension1D;
    return ret;
};
imports.wbg.__wbg_maxTextureDimension2D_26995ffa94733f82 = function(arg0) {
    const ret = getObject(arg0).maxTextureDimension2D;
    return ret;
};
imports.wbg.__wbg_maxTextureDimension3D_8d77c6d768caef58 = function(arg0) {
    const ret = getObject(arg0).maxTextureDimension3D;
    return ret;
};
imports.wbg.__wbg_maxTextureArrayLayers_cbf7e90284df66c3 = function(arg0) {
    const ret = getObject(arg0).maxTextureArrayLayers;
    return ret;
};
imports.wbg.__wbg_maxBindGroups_54fa38a646718d85 = function(arg0) {
    const ret = getObject(arg0).maxBindGroups;
    return ret;
};
imports.wbg.__wbg_maxBindingsPerBindGroup_e8f7a2792b9ac107 = function(arg0) {
    const ret = getObject(arg0).maxBindingsPerBindGroup;
    return ret;
};
imports.wbg.__wbg_maxDynamicUniformBuffersPerPipelineLayout_7c5942f359a6fb1b = function(arg0) {
    const ret = getObject(arg0).maxDynamicUniformBuffersPerPipelineLayout;
    return ret;
};
imports.wbg.__wbg_maxDynamicStorageBuffersPerPipelineLayout_bd22a382d13e6ef5 = function(arg0) {
    const ret = getObject(arg0).maxDynamicStorageBuffersPerPipelineLayout;
    return ret;
};
imports.wbg.__wbg_maxSampledTexturesPerShaderStage_5704d5ff400bceee = function(arg0) {
    const ret = getObject(arg0).maxSampledTexturesPerShaderStage;
    return ret;
};
imports.wbg.__wbg_maxSamplersPerShaderStage_5e8845f07c33913a = function(arg0) {
    const ret = getObject(arg0).maxSamplersPerShaderStage;
    return ret;
};
imports.wbg.__wbg_maxStorageBuffersPerShaderStage_18a674788ed5fdad = function(arg0) {
    const ret = getObject(arg0).maxStorageBuffersPerShaderStage;
    return ret;
};
imports.wbg.__wbg_maxStorageTexturesPerShaderStage_bfff5cb8d91bcfcc = function(arg0) {
    const ret = getObject(arg0).maxStorageTexturesPerShaderStage;
    return ret;
};
imports.wbg.__wbg_maxUniformBuffersPerShaderStage_ef06df9be2943d45 = function(arg0) {
    const ret = getObject(arg0).maxUniformBuffersPerShaderStage;
    return ret;
};
imports.wbg.__wbg_maxUniformBufferBindingSize_f84670235a7e5df9 = function(arg0) {
    const ret = getObject(arg0).maxUniformBufferBindingSize;
    return ret;
};
imports.wbg.__wbg_maxStorageBufferBindingSize_9245cd89c719dbf2 = function(arg0) {
    const ret = getObject(arg0).maxStorageBufferBindingSize;
    return ret;
};
imports.wbg.__wbg_minUniformBufferOffsetAlignment_5574ef5e4f6d62da = function(arg0) {
    const ret = getObject(arg0).minUniformBufferOffsetAlignment;
    return ret;
};
imports.wbg.__wbg_minStorageBufferOffsetAlignment_a6666e346184b953 = function(arg0) {
    const ret = getObject(arg0).minStorageBufferOffsetAlignment;
    return ret;
};
imports.wbg.__wbg_maxVertexBuffers_73da155813feea78 = function(arg0) {
    const ret = getObject(arg0).maxVertexBuffers;
    return ret;
};
imports.wbg.__wbg_maxBufferSize_7087869d4548c87d = function(arg0) {
    const ret = getObject(arg0).maxBufferSize;
    return ret;
};
imports.wbg.__wbg_maxVertexAttributes_3a0ea01143239608 = function(arg0) {
    const ret = getObject(arg0).maxVertexAttributes;
    return ret;
};
imports.wbg.__wbg_maxVertexBufferArrayStride_d699c03944dd52d9 = function(arg0) {
    const ret = getObject(arg0).maxVertexBufferArrayStride;
    return ret;
};
imports.wbg.__wbg_maxInterStageShaderComponents_09be6edd346cb8da = function(arg0) {
    const ret = getObject(arg0).maxInterStageShaderComponents;
    return ret;
};
imports.wbg.__wbg_maxColorAttachments_bacf53ca2877dee7 = function(arg0) {
    const ret = getObject(arg0).maxColorAttachments;
    return ret;
};
imports.wbg.__wbg_maxColorAttachmentBytesPerSample_38cc3bda21b1c9da = function(arg0) {
    const ret = getObject(arg0).maxColorAttachmentBytesPerSample;
    return ret;
};
imports.wbg.__wbg_maxComputeWorkgroupStorageSize_58415be93e502f25 = function(arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupStorageSize;
    return ret;
};
imports.wbg.__wbg_maxComputeInvocationsPerWorkgroup_8aa2f0a5861ce5ef = function(arg0) {
    const ret = getObject(arg0).maxComputeInvocationsPerWorkgroup;
    return ret;
};
imports.wbg.__wbg_maxComputeWorkgroupSizeX_789174905500f6c7 = function(arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupSizeX;
    return ret;
};
imports.wbg.__wbg_maxComputeWorkgroupSizeY_926ec1c24c6136da = function(arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupSizeY;
    return ret;
};
imports.wbg.__wbg_maxComputeWorkgroupSizeZ_562c888ae9402be1 = function(arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupSizeZ;
    return ret;
};
imports.wbg.__wbg_maxComputeWorkgroupsPerDimension_07fa50cdca40e120 = function(arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupsPerDimension;
    return ret;
};
imports.wbg.__wbg_error_7ced2e8034eb1f3f = function(arg0) {
    const ret = getObject(arg0).error;
    return addHeapObject(ret);
};
imports.wbg.__wbg_message_867097f776344069 = function(arg0, arg1) {
    const ret = getObject(arg1).message;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbindgen_is_object = function(arg0) {
    const val = getObject(arg0);
    const ret = typeof(val) === 'object' && val !== null;
    return ret;
};
imports.wbg.__wbg_getPreferredCanvasFormat_c57006806f2efe1b = function(arg0) {
    const ret = getObject(arg0).getPreferredCanvasFormat();
    return addHeapObject(ret);
};
imports.wbg.__wbg_requestAdapter_913357b9788f14cd = function(arg0, arg1) {
    const ret = getObject(arg0).requestAdapter(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_features_01f848ca4efe700b = function(arg0) {
    const ret = getObject(arg0).features;
    return addHeapObject(ret);
};
imports.wbg.__wbg_limits_cf6e9ab92d696f0c = function(arg0) {
    const ret = getObject(arg0).limits;
    return addHeapObject(ret);
};
imports.wbg.__wbg_queue_9f8d8658085c6f43 = function(arg0) {
    const ret = getObject(arg0).queue;
    return addHeapObject(ret);
};
imports.wbg.__wbg_lost_b77b9d55ecb164aa = function(arg0) {
    const ret = getObject(arg0).lost;
    return addHeapObject(ret);
};
imports.wbg.__wbg_setonuncapturederror_0901d4d8bff41810 = function(arg0, arg1) {
    getObject(arg0).onuncapturederror = getObject(arg1);
};
imports.wbg.__wbg_createBindGroup_5ac37963cb812b24 = function(arg0, arg1) {
    const ret = getObject(arg0).createBindGroup(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_createBindGroupLayout_6adcd872318d899a = function(arg0, arg1) {
    const ret = getObject(arg0).createBindGroupLayout(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_createBuffer_90ac080c7cc1375d = function(arg0, arg1) {
    const ret = getObject(arg0).createBuffer(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_createCommandEncoder_9ee63be2a93c77dd = function(arg0, arg1) {
    const ret = getObject(arg0).createCommandEncoder(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_createComputePipeline_957ea1dbcd97e6de = function(arg0, arg1) {
    const ret = getObject(arg0).createComputePipeline(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_createPipelineLayout_2648fbc756354294 = function(arg0, arg1) {
    const ret = getObject(arg0).createPipelineLayout(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_createQuerySet_c6b5390470139efb = function(arg0, arg1) {
    const ret = getObject(arg0).createQuerySet(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_createRenderBundleEncoder_bbce060a45e55caf = function(arg0, arg1) {
    const ret = getObject(arg0).createRenderBundleEncoder(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_createRenderPipeline_513576fa326b8ccf = function(arg0, arg1) {
    const ret = getObject(arg0).createRenderPipeline(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_createSampler_e56450d56435986f = function(arg0, arg1) {
    const ret = getObject(arg0).createSampler(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_createShaderModule_6851cf2067c2f947 = function(arg0, arg1) {
    const ret = getObject(arg0).createShaderModule(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_createTexture_4297303d703376ef = function(arg0, arg1) {
    const ret = getObject(arg0).createTexture(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_destroy_6e1daab7792230a0 = function(arg0) {
    getObject(arg0).destroy();
};
imports.wbg.__wbg_popErrorScope_1d998d85c7b134be = function(arg0) {
    const ret = getObject(arg0).popErrorScope();
    return addHeapObject(ret);
};
imports.wbg.__wbg_pushErrorScope_d39727ef0414ac9f = function(arg0, arg1) {
    getObject(arg0).pushErrorScope(takeObject(arg1));
};
imports.wbg.__wbg_copyExternalImageToTexture_5389ee5babf9d86f = function(arg0, arg1, arg2, arg3) {
    getObject(arg0).copyExternalImageToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
};
imports.wbg.__wbg_submit_c512d9a4b5ff838d = function(arg0, arg1) {
    getObject(arg0).submit(getObject(arg1));
};
imports.wbg.__wbg_writeBuffer_b225dafa1a52c298 = function(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).writeBuffer(getObject(arg1), arg2, getObject(arg3), arg4, arg5);
};
imports.wbg.__wbg_writeTexture_05b125d21ce9740e = function(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).writeTexture(getObject(arg1), getObject(arg2), getObject(arg3), getObject(arg4));
};
imports.wbg.__wbg_gpu_4ac835f782ad971d = function(arg0) {
    const ret = getObject(arg0).gpu;
    return addHeapObject(ret);
};
imports.wbg.__wbg_size_7838da1244dcc49f = function(arg0) {
    const ret = getObject(arg0).size;
    return ret;
};
imports.wbg.__wbg_usage_2e5ff7c87b5e9737 = function(arg0) {
    const ret = getObject(arg0).usage;
    return ret;
};
imports.wbg.__wbg_destroy_b8ea7d8b8cee78c4 = function(arg0) {
    getObject(arg0).destroy();
};
imports.wbg.__wbg_getMappedRange_8229b08f744819c0 = function(arg0, arg1, arg2) {
    const ret = getObject(arg0).getMappedRange(arg1, arg2);
    return addHeapObject(ret);
};
imports.wbg.__wbg_mapAsync_7d9fc5c22fb1f55e = function(arg0, arg1, arg2, arg3) {
    const ret = getObject(arg0).mapAsync(arg1 >>> 0, arg2, arg3);
    return addHeapObject(ret);
};
imports.wbg.__wbg_unmap_abe29e47be94736f = function(arg0) {
    getObject(arg0).unmap();
};
imports.wbg.__wbg_instanceof_GpuAdapter_76bb05881d5f91d1 = function(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUAdapter;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};
imports.wbg.__wbg_features_7fd6ee02e18d77a4 = function(arg0) {
    const ret = getObject(arg0).features;
    return addHeapObject(ret);
};
imports.wbg.__wbg_limits_7c1e17ce28ddf954 = function(arg0) {
    const ret = getObject(arg0).limits;
    return addHeapObject(ret);
};
imports.wbg.__wbg_requestDevice_baf0b46015a90431 = function(arg0, arg1) {
    const ret = getObject(arg0).requestDevice(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_label_4956528ad99b1650 = function(arg0, arg1) {
    const ret = getObject(arg1).label;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_beginComputePass_99e2aa27fb960fa5 = function(arg0, arg1) {
    const ret = getObject(arg0).beginComputePass(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_beginRenderPass_b4c178a1fd787b5c = function(arg0, arg1) {
    const ret = getObject(arg0).beginRenderPass(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_clearBuffer_50e1d3d029849fdb = function(arg0, arg1, arg2) {
    getObject(arg0).clearBuffer(getObject(arg1), arg2);
};
imports.wbg.__wbg_clearBuffer_157bab025583c473 = function(arg0, arg1, arg2, arg3) {
    getObject(arg0).clearBuffer(getObject(arg1), arg2, arg3);
};
imports.wbg.__wbg_copyBufferToBuffer_0a44e23b31a7ca5a = function(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).copyBufferToBuffer(getObject(arg1), arg2, getObject(arg3), arg4, arg5);
};
imports.wbg.__wbg_copyBufferToTexture_de6f3cd9ac87a870 = function(arg0, arg1, arg2, arg3) {
    getObject(arg0).copyBufferToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
};
imports.wbg.__wbg_copyTextureToBuffer_7ab49ff0dd12cd22 = function(arg0, arg1, arg2, arg3) {
    getObject(arg0).copyTextureToBuffer(getObject(arg1), getObject(arg2), getObject(arg3));
};
imports.wbg.__wbg_copyTextureToTexture_45800f5fb0aaaf6c = function(arg0, arg1, arg2, arg3) {
    getObject(arg0).copyTextureToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
};
imports.wbg.__wbg_finish_3cd844105a9de3e9 = function(arg0) {
    const ret = getObject(arg0).finish();
    return addHeapObject(ret);
};
imports.wbg.__wbg_finish_cbd8e5d52fe81fd6 = function(arg0, arg1) {
    const ret = getObject(arg0).finish(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_resolveQuerySet_8f696a33e8da099f = function(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).resolveQuerySet(getObject(arg1), arg2 >>> 0, arg3 >>> 0, getObject(arg4), arg5 >>> 0);
};
imports.wbg.__wbg_finish_806df42c71c712c3 = function(arg0) {
    const ret = getObject(arg0).finish();
    return addHeapObject(ret);
};
imports.wbg.__wbg_finish_55ef253db8a2e02a = function(arg0, arg1) {
    const ret = getObject(arg0).finish(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_setBindGroup_2054136f79b0fed9 = function(arg0, arg1, arg2) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
};
imports.wbg.__wbg_setBindGroup_7908d39626c7bcc5 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
};
imports.wbg.__wbg_draw_2ea14b17b7ad7b86 = function(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
};
imports.wbg.__wbg_drawIndexed_81f7662bc9f8bda1 = function(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
};
imports.wbg.__wbg_drawIndexedIndirect_74e31bc5d14e7aab = function(arg0, arg1, arg2) {
    getObject(arg0).drawIndexedIndirect(getObject(arg1), arg2);
};
imports.wbg.__wbg_drawIndirect_3de3a4df802f8f74 = function(arg0, arg1, arg2) {
    getObject(arg0).drawIndirect(getObject(arg1), arg2);
};
imports.wbg.__wbg_setIndexBuffer_4deca629ec05a510 = function(arg0, arg1, arg2, arg3) {
    getObject(arg0).setIndexBuffer(getObject(arg1), takeObject(arg2), arg3);
};
imports.wbg.__wbg_setIndexBuffer_ea5677e397c8df89 = function(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setIndexBuffer(getObject(arg1), takeObject(arg2), arg3, arg4);
};
imports.wbg.__wbg_setPipeline_b1e4ff4a2d89b8aa = function(arg0, arg1) {
    getObject(arg0).setPipeline(getObject(arg1));
};
imports.wbg.__wbg_setVertexBuffer_4c924a9cc335e437 = function(arg0, arg1, arg2, arg3) {
    getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3);
};
imports.wbg.__wbg_setVertexBuffer_0aca41ad007e04fc = function(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3, arg4);
};
imports.wbg.__wbg_createView_8463cbef5f0c4d5c = function(arg0, arg1) {
    const ret = getObject(arg0).createView(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_destroy_7fe69567d342b339 = function(arg0) {
    getObject(arg0).destroy();
};
imports.wbg.__wbg_instanceof_GpuValidationError_3128431f7a0514f4 = function(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUValidationError;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};
imports.wbg.__wbg_getBindGroupLayout_255eaa69c120a995 = function(arg0, arg1) {
    const ret = getObject(arg0).getBindGroupLayout(arg1 >>> 0);
    return addHeapObject(ret);
};
imports.wbg.__wbg_getBindGroupLayout_d573a4d2adfb5ae8 = function(arg0, arg1) {
    const ret = getObject(arg0).getBindGroupLayout(arg1 >>> 0);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_number_get = function(arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof(obj) === 'number' ? obj : undefined;
    getFloat64Memory0()[arg0 / 8 + 1] = isLikeNone(ret) ? 0 : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
};
imports.wbg.__wbindgen_boolean_get = function(arg0) {
    const v = getObject(arg0);
    const ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
    return ret;
};
imports.wbg.__wbindgen_jsval_eq = function(arg0, arg1) {
    const ret = getObject(arg0) === getObject(arg1);
    return ret;
};
imports.wbg.__wbg_instanceof_Window_f401953a2cf86220 = function(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Window;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};
imports.wbg.__wbg_document_5100775d18896c16 = function(arg0) {
    const ret = getObject(arg0).document;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_location_2951b5ee34f19221 = function(arg0) {
    const ret = getObject(arg0).location;
    return addHeapObject(ret);
};
imports.wbg.__wbg_history_bc4057de66a2015f = function() { return handleError(function (arg0) {
    const ret = getObject(arg0).history;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_navigator_6c8fa55c5cc8796e = function(arg0) {
    const ret = getObject(arg0).navigator;
    return addHeapObject(ret);
};
imports.wbg.__wbg_devicePixelRatio_efc553b59506f64c = function(arg0) {
    const ret = getObject(arg0).devicePixelRatio;
    return ret;
};
imports.wbg.__wbg_cancelIdleCallback_3a36cf77475b492b = function(arg0, arg1) {
    getObject(arg0).cancelIdleCallback(arg1 >>> 0);
};
imports.wbg.__wbg_getComputedStyle_078292ffe423aded = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).getComputedStyle(getObject(arg1));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_matchMedia_66bb21e3ef19270c = function() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).matchMedia(v0);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_requestIdleCallback_cee8e1d6bdcfae9e = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).requestIdleCallback(getObject(arg1));
    return ret;
}, arguments) };
imports.wbg.__wbg_scrollTo_4d970c5e1c4b340b = function(arg0, arg1, arg2) {
    getObject(arg0).scrollTo(arg1, arg2);
};
imports.wbg.__wbg_cancelAnimationFrame_111532f326e480af = function() { return handleError(function (arg0, arg1) {
    getObject(arg0).cancelAnimationFrame(arg1);
}, arguments) };
imports.wbg.__wbg_requestAnimationFrame_549258cfa66011f0 = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
    return ret;
}, arguments) };
imports.wbg.__wbg_clearTimeout_ba63ae54a36e111e = function(arg0, arg1) {
    getObject(arg0).clearTimeout(arg1);
};
imports.wbg.__wbg_setTimeout_d2b9a986d10a6182 = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).setTimeout(getObject(arg1));
    return ret;
}, arguments) };
imports.wbg.__wbg_setTimeout_c172d5704ef82276 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
    return ret;
}, arguments) };
imports.wbg.__wbg_documentElement_da9c841ddb352d95 = function(arg0) {
    const ret = getObject(arg0).documentElement;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_body_edb1908d3ceff3a1 = function(arg0) {
    const ret = getObject(arg0).body;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_visibilityState_990071edf70b1c55 = function(arg0) {
    const ret = getObject(arg0).visibilityState;
    return addHeapObject(ret);
};
imports.wbg.__wbg_activeElement_fa7feca08f5028c0 = function(arg0) {
    const ret = getObject(arg0).activeElement;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_fullscreenElement_1bef71098bd8dfde = function(arg0) {
    const ret = getObject(arg0).fullscreenElement;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_createComment_354ccab4fdc521ee = function(arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).createComment(v0);
    return addHeapObject(ret);
};
imports.wbg.__wbg_createDocumentFragment_8c86903bbb0a3c3c = function(arg0) {
    const ret = getObject(arg0).createDocumentFragment();
    return addHeapObject(ret);
};
imports.wbg.__wbg_createElement_8bae7856a4bb7411 = function() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).createElement(v0);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_exitFullscreen_5679ad2b002921bd = function(arg0) {
    getObject(arg0).exitFullscreen();
};
imports.wbg.__wbg_getElementById_c369ff43f0db99cf = function(arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).getElementById(v0);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_querySelectorAll_4e0fcdb64cda2cd5 = function() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).querySelectorAll(v0);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_altKey_2e6c34c37088d8b1 = function(arg0) {
    const ret = getObject(arg0).altKey;
    return ret;
};
imports.wbg.__wbg_ctrlKey_bb5b6fef87339703 = function(arg0) {
    const ret = getObject(arg0).ctrlKey;
    return ret;
};
imports.wbg.__wbg_shiftKey_5911baf439ab232b = function(arg0) {
    const ret = getObject(arg0).shiftKey;
    return ret;
};
imports.wbg.__wbg_metaKey_6bf4ae4e83a11278 = function(arg0) {
    const ret = getObject(arg0).metaKey;
    return ret;
};
imports.wbg.__wbg_location_f7b033ddfc516739 = function(arg0) {
    const ret = getObject(arg0).location;
    return ret;
};
imports.wbg.__wbg_repeat_f64b916c6eed0685 = function(arg0) {
    const ret = getObject(arg0).repeat;
    return ret;
};
imports.wbg.__wbg_key_dccf9e8aa1315a8e = function(arg0, arg1) {
    const ret = getObject(arg1).key;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_code_3b0c3912a2351163 = function(arg0, arg1) {
    const ret = getObject(arg1).code;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_pointerId_e030fa156647fedd = function(arg0) {
    const ret = getObject(arg0).pointerId;
    return ret;
};
imports.wbg.__wbg_pressure_99cd07399f942a7c = function(arg0) {
    const ret = getObject(arg0).pressure;
    return ret;
};
imports.wbg.__wbg_pointerType_0f2f0383406aa7fa = function(arg0, arg1) {
    const ret = getObject(arg1).pointerType;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_getCoalescedEvents_14b443b6f75837a2 = function(arg0) {
    const ret = getObject(arg0).getCoalescedEvents();
    return addHeapObject(ret);
};
imports.wbg.__wbg_view_7f0ce470793a340f = function(arg0) {
    const ret = getObject(arg0).view;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_respond_b1a43b2e3a06d525 = function() { return handleError(function (arg0, arg1) {
    getObject(arg0).respond(arg1 >>> 0);
}, arguments) };
imports.wbg.__wbg_contentRect_bce644376332c7a5 = function(arg0) {
    const ret = getObject(arg0).contentRect;
    return addHeapObject(ret);
};
imports.wbg.__wbg_borderBoxSize_d8bfbc0ce20f941d = function(arg0) {
    const ret = getObject(arg0).borderBoxSize;
    return addHeapObject(ret);
};
imports.wbg.__wbg_contentBoxSize_c6294de1f1ca7e41 = function(arg0) {
    const ret = getObject(arg0).contentBoxSize;
    return addHeapObject(ret);
};
imports.wbg.__wbg_devicePixelContentBoxSize_d5bcdcd5e96671f3 = function(arg0) {
    const ret = getObject(arg0).devicePixelContentBoxSize;
    return addHeapObject(ret);
};
imports.wbg.__wbg_error_8e3928cfb8a43e2b = function(arg0) {
    console.error(getObject(arg0));
};
imports.wbg.__wbg_error_6e987ee48d9fdf45 = function(arg0, arg1) {
    console.error(getObject(arg0), getObject(arg1));
};
imports.wbg.__wbg_warn_63bbae1730aead09 = function(arg0) {
    console.warn(getObject(arg0));
};
imports.wbg.__wbg_namespaceURI_5235ee79fd5f6781 = function(arg0, arg1) {
    const ret = getObject(arg1).namespaceURI;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_setinnerHTML_26d69b59e1af99c7 = function(arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).innerHTML = v0;
};
imports.wbg.__wbg_outerHTML_e073aa84e7bc1eaf = function(arg0, arg1) {
    const ret = getObject(arg1).outerHTML;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_getAttribute_99bddb29274b29b9 = function(arg0, arg1, arg2, arg3) {
    var v0 = getCachedStringFromWasm0(arg2, arg3);
    const ret = getObject(arg1).getAttribute(v0);
    var ptr2 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len2 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len2;
    getInt32Memory0()[arg0 / 4 + 0] = ptr2;
};
imports.wbg.__wbg_hasAttribute_8340e1a2a46f10f3 = function(arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).hasAttribute(v0);
    return ret;
};
imports.wbg.__wbg_removeAttribute_1b10a06ae98ebbd1 = function() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).removeAttribute(v0);
}, arguments) };
imports.wbg.__wbg_scrollIntoView_0c1a31f3d0dce6ae = function(arg0) {
    getObject(arg0).scrollIntoView();
};
imports.wbg.__wbg_setAttribute_3c9f6c303b696daa = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    var v1 = getCachedStringFromWasm0(arg3, arg4);
    getObject(arg0).setAttribute(v0, v1);
}, arguments) };
imports.wbg.__wbg_setPointerCapture_0fdaad7a916c8486 = function() { return handleError(function (arg0, arg1) {
    getObject(arg0).setPointerCapture(arg1);
}, arguments) };
imports.wbg.__wbg_before_210596e44d88649f = function() { return handleError(function (arg0, arg1) {
    getObject(arg0).before(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_remove_49b0a5925a04b955 = function(arg0) {
    getObject(arg0).remove();
};
imports.wbg.__wbg_defaultPrevented_cc14a1dd3dd69c38 = function(arg0) {
    const ret = getObject(arg0).defaultPrevented;
    return ret;
};
imports.wbg.__wbg_composedPath_58473fd5ae55f2cd = function(arg0) {
    const ret = getObject(arg0).composedPath();
    return addHeapObject(ret);
};
imports.wbg.__wbg_preventDefault_b1a4aafc79409429 = function(arg0) {
    getObject(arg0).preventDefault();
};
imports.wbg.__wbg_new_4e95a9abecc83cd4 = function() { return handleError(function (arg0) {
    const ret = new IntersectionObserver(getObject(arg0));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_disconnect_e694940ce6d0ef91 = function(arg0) {
    getObject(arg0).disconnect();
};
imports.wbg.__wbg_observe_538a6d1df0deb993 = function(arg0, arg1) {
    getObject(arg0).observe(getObject(arg1));
};
imports.wbg.__wbg_isIntersecting_082397a1d66e2e35 = function(arg0) {
    const ret = getObject(arg0).isIntersecting;
    return ret;
};
imports.wbg.__wbg_childNodes_118168e8b23bcb9b = function(arg0) {
    const ret = getObject(arg0).childNodes;
    return addHeapObject(ret);
};
imports.wbg.__wbg_previousSibling_9708a091a3e6e03b = function(arg0) {
    const ret = getObject(arg0).previousSibling;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_nextSibling_709614fdb0fb7a66 = function(arg0) {
    const ret = getObject(arg0).nextSibling;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_textContent_8e392d624539e731 = function(arg0, arg1) {
    const ret = getObject(arg1).textContent;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_settextContent_d271bab459cbb1ba = function(arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).textContent = v0;
};
imports.wbg.__wbg_appendChild_580ccb11a660db68 = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).appendChild(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_cloneNode_e19c313ea20d5d1d = function() { return handleError(function (arg0) {
    const ret = getObject(arg0).cloneNode();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_contains_fdfd1dc667f36695 = function(arg0, arg1) {
    const ret = getObject(arg0).contains(getObject(arg1));
    return ret;
};
imports.wbg.__wbg_href_7bfb3b2fdc0a6c3f = function(arg0, arg1) {
    const ret = getObject(arg1).href;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_origin_ea68ac578fa8517a = function(arg0, arg1) {
    const ret = getObject(arg1).origin;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_pathname_c5fe403ef9525ec6 = function(arg0, arg1) {
    const ret = getObject(arg1).pathname;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_search_c68f506c44be6d1e = function(arg0, arg1) {
    const ret = getObject(arg1).search;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_searchParams_bc5845fe67587f77 = function(arg0) {
    const ret = getObject(arg0).searchParams;
    return addHeapObject(ret);
};
imports.wbg.__wbg_hash_cdea7a9b7e684a42 = function(arg0, arg1) {
    const ret = getObject(arg1).hash;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_newwithbase_6aabbfb1b2e6a1cb = function() { return handleError(function (arg0, arg1, arg2, arg3) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
    var v1 = getCachedStringFromWasm0(arg2, arg3);
    const ret = new URL(v0, v1);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_instanceof_HtmlAnchorElement_5fc0eb2fbc8672d8 = function(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof HTMLAnchorElement;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};
imports.wbg.__wbg_target_f0876f510847bc60 = function(arg0, arg1) {
    const ret = getObject(arg1).target;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_href_40fd5bca11c13133 = function(arg0, arg1) {
    const ret = getObject(arg1).href;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_persisted_cbb7e3c657029516 = function(arg0) {
    const ret = getObject(arg0).persisted;
    return ret;
};
imports.wbg.__wbg_state_9cc3f933b7d50acb = function() { return handleError(function (arg0) {
    const ret = getObject(arg0).state;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_pushState_b8e8d346f8bb33fd = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    var v0 = getCachedStringFromWasm0(arg2, arg3);
    var v1 = getCachedStringFromWasm0(arg4, arg5);
    getObject(arg0).pushState(getObject(arg1), v0, v1);
}, arguments) };
imports.wbg.__wbg_replaceState_ec9431bea5108a50 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    var v0 = getCachedStringFromWasm0(arg2, arg3);
    var v1 = getCachedStringFromWasm0(arg4, arg5);
    getObject(arg0).replaceState(getObject(arg1), v0, v1);
}, arguments) };
imports.wbg.__wbg_setonmessage_93bdba94dcd46c04 = function(arg0, arg1) {
    getObject(arg0).onmessage = getObject(arg1);
};
imports.wbg.__wbg_close_a5883ed21dc3d115 = function(arg0) {
    getObject(arg0).close();
};
imports.wbg.__wbg_postMessage_fbddfe9314af804e = function() { return handleError(function (arg0, arg1) {
    getObject(arg0).postMessage(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_start_5a293222bc398f51 = function(arg0) {
    getObject(arg0).start();
};
imports.wbg.__wbg_width_1e8430024cb82aba = function(arg0) {
    const ret = getObject(arg0).width;
    return ret;
};
imports.wbg.__wbg_height_0c1394f089d7bb71 = function(arg0) {
    const ret = getObject(arg0).height;
    return ret;
};
imports.wbg.__wbg_sethref_b94692d1a9f05b53 = function() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).href = v0;
}, arguments) };
imports.wbg.__wbg_origin_ee93e29ace71f568 = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg1).origin;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_pathname_5449afe3829f96a1 = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg1).pathname;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_search_489f12953342ec1f = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg1).search;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_hash_553098e838e06c1d = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg1).hash;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };
imports.wbg.__wbg_close_a994f9425dab445c = function() { return handleError(function (arg0) {
    getObject(arg0).close();
}, arguments) };
imports.wbg.__wbg_enqueue_ea194723156c0cc2 = function() { return handleError(function (arg0, arg1) {
    getObject(arg0).enqueue(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_signal_a61f78a3478fd9bc = function(arg0) {
    const ret = getObject(arg0).signal;
    return addHeapObject(ret);
};
imports.wbg.__wbg_new_0d76b0581eca6298 = function() { return handleError(function () {
    const ret = new AbortController();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_abort_2aa7521d5690750e = function(arg0) {
    getObject(arg0).abort();
};
imports.wbg.__wbg_addEventListener_53b787075bd5e003 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).addEventListener(v0, getObject(arg3));
}, arguments) };
imports.wbg.__wbg_removeEventListener_92cb9b3943463338 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).removeEventListener(v0, getObject(arg3));
}, arguments) };
imports.wbg.__wbg_deltaX_206576827ededbe5 = function(arg0) {
    const ret = getObject(arg0).deltaX;
    return ret;
};
imports.wbg.__wbg_deltaY_032e327e216f2b2b = function(arg0) {
    const ret = getObject(arg0).deltaY;
    return ret;
};
imports.wbg.__wbg_deltaMode_294b2eaf54047265 = function(arg0) {
    const ret = getObject(arg0).deltaMode;
    return ret;
};
imports.wbg.__wbg_setdata_8c2b43af041cc1b3 = function(arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    getObject(arg0).data = v0;
};
imports.wbg.__wbg_media_bcef0e2ec4383569 = function(arg0, arg1) {
    const ret = getObject(arg1).media;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbg_matches_e14ed9ff8291cf24 = function(arg0) {
    const ret = getObject(arg0).matches;
    return ret;
};
imports.wbg.__wbg_addListener_143ad0a501fabc3a = function() { return handleError(function (arg0, arg1) {
    getObject(arg0).addListener(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_removeListener_46f3ee00c5b95320 = function() { return handleError(function (arg0, arg1) {
    getObject(arg0).removeListener(getObject(arg1));
}, arguments) };
imports.wbg.__wbg_inlineSize_ff0e40258cefeba2 = function(arg0) {
    const ret = getObject(arg0).inlineSize;
    return ret;
};
imports.wbg.__wbg_blockSize_73f4e5608c08713d = function(arg0) {
    const ret = getObject(arg0).blockSize;
    return ret;
};
imports.wbg.__wbg_append_f7fa3534fc158323 = function() { return handleError(function (arg0, arg1, arg2) {
    getObject(arg0).append(getObject(arg1), getObject(arg2));
}, arguments) };
imports.wbg.__wbg_length_d0a802565d17eec4 = function(arg0) {
    const ret = getObject(arg0).length;
    return ret;
};
imports.wbg.__wbg_get_8cd5eba00ab6304f = function(arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_new_61d4f20a1c08a45c = function() { return handleError(function (arg0) {
    const ret = new ResizeObserver(getObject(arg0));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_disconnect_6675f32e2ae8deb7 = function(arg0) {
    getObject(arg0).disconnect();
};
imports.wbg.__wbg_observe_a79646ce7bb08cb8 = function(arg0, arg1) {
    getObject(arg0).observe(getObject(arg1));
};
imports.wbg.__wbg_observe_dc0ebcd59ee7cd17 = function(arg0, arg1, arg2) {
    getObject(arg0).observe(getObject(arg1), getObject(arg2));
};
imports.wbg.__wbg_unobserve_55c93518cad6ac06 = function(arg0, arg1) {
    getObject(arg0).unobserve(getObject(arg1));
};
imports.wbg.__wbg_width_aee8b8809b033b05 = function(arg0) {
    const ret = getObject(arg0).width;
    return ret;
};
imports.wbg.__wbg_setwidth_080107476e633963 = function(arg0, arg1) {
    getObject(arg0).width = arg1 >>> 0;
};
imports.wbg.__wbg_height_80053d3c71b338e0 = function(arg0) {
    const ret = getObject(arg0).height;
    return ret;
};
imports.wbg.__wbg_setheight_dc240617639f1f51 = function(arg0, arg1) {
    getObject(arg0).height = arg1 >>> 0;
};
imports.wbg.__wbg_getContext_df50fa48a8876636 = function() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).getContext(v0);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_port1_d51a1bd2c33125d0 = function(arg0) {
    const ret = getObject(arg0).port1;
    return addHeapObject(ret);
};
imports.wbg.__wbg_port2_f522a81e92362e7e = function(arg0) {
    const ret = getObject(arg0).port2;
    return addHeapObject(ret);
};
imports.wbg.__wbg_new_34615e164dc78975 = function() { return handleError(function () {
    const ret = new MessageChannel();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_ctrlKey_008695ce60a588f5 = function(arg0) {
    const ret = getObject(arg0).ctrlKey;
    return ret;
};
imports.wbg.__wbg_shiftKey_1e76dbfcdd36a4b4 = function(arg0) {
    const ret = getObject(arg0).shiftKey;
    return ret;
};
imports.wbg.__wbg_altKey_07da841b54bd3ed6 = function(arg0) {
    const ret = getObject(arg0).altKey;
    return ret;
};
imports.wbg.__wbg_metaKey_86bfd3b0d3a8083f = function(arg0) {
    const ret = getObject(arg0).metaKey;
    return ret;
};
imports.wbg.__wbg_button_367cdc7303e3cf9b = function(arg0) {
    const ret = getObject(arg0).button;
    return ret;
};
imports.wbg.__wbg_buttons_d004fa75ac704227 = function(arg0) {
    const ret = getObject(arg0).buttons;
    return ret;
};
imports.wbg.__wbg_movementX_b800a0cacd14d9bf = function(arg0) {
    const ret = getObject(arg0).movementX;
    return ret;
};
imports.wbg.__wbg_movementY_7907e03eb8c0ea1e = function(arg0) {
    const ret = getObject(arg0).movementY;
    return ret;
};
imports.wbg.__wbg_getPropertyValue_fa32ee1811f224cb = function() { return handleError(function (arg0, arg1, arg2, arg3) {
    var v0 = getCachedStringFromWasm0(arg2, arg3);
    const ret = getObject(arg1).getPropertyValue(v0);
    const ptr2 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len2 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len2;
    getInt32Memory0()[arg0 / 4 + 0] = ptr2;
}, arguments) };
imports.wbg.__wbg_removeProperty_fa6d48e2923dcfac = function() { return handleError(function (arg0, arg1, arg2, arg3) {
    var v0 = getCachedStringFromWasm0(arg2, arg3);
    const ret = getObject(arg1).removeProperty(v0);
    const ptr2 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len2 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len2;
    getInt32Memory0()[arg0 / 4 + 0] = ptr2;
}, arguments) };
imports.wbg.__wbg_setProperty_ea7d15a2b591aa97 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    var v1 = getCachedStringFromWasm0(arg3, arg4);
    getObject(arg0).setProperty(v0, v1);
}, arguments) };
imports.wbg.__wbg_setwidth_83d936c4b04dcbec = function(arg0, arg1) {
    getObject(arg0).width = arg1 >>> 0;
};
imports.wbg.__wbg_setheight_6025ba0d58e6cc8c = function(arg0, arg1) {
    getObject(arg0).height = arg1 >>> 0;
};
imports.wbg.__wbg_getContext_c102f659d540d068 = function() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).getContext(v0);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_navigator_56803b85352a0575 = function(arg0) {
    const ret = getObject(arg0).navigator;
    return addHeapObject(ret);
};
imports.wbg.__wbg_instanceof_HtmlElement_3bcc4ff70cfdcba5 = function(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof HTMLElement;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};
imports.wbg.__wbg_style_c3fc3dd146182a2d = function(arg0) {
    const ret = getObject(arg0).style;
    return addHeapObject(ret);
};
imports.wbg.__wbg_focus_39d4b8ba8ff9df14 = function() { return handleError(function (arg0) {
    getObject(arg0).focus();
}, arguments) };
imports.wbg.__wbg_byobRequest_72fca99f9c32c193 = function(arg0) {
    const ret = getObject(arg0).byobRequest;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_close_184931724d961ccc = function() { return handleError(function (arg0) {
    getObject(arg0).close();
}, arguments) };
imports.wbg.__wbindgen_is_function = function(arg0) {
    const ret = typeof(getObject(arg0)) === 'function';
    return ret;
};
imports.wbg.__wbg_queueMicrotask_481971b0d87f3dd4 = function(arg0) {
    queueMicrotask(getObject(arg0));
};
imports.wbg.__wbg_queueMicrotask_3cbae2ec6b6cd3d6 = function(arg0) {
    const ret = getObject(arg0).queueMicrotask;
    return addHeapObject(ret);
};
imports.wbg.__wbg_decodeURI_34e1afc7326c927c = function() { return handleError(function (arg0, arg1) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
    const ret = decodeURI(v0);
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_new_16b304a2cfa7ff4a = function() {
    const ret = new Array();
    return addHeapObject(ret);
};
imports.wbg.__wbg_get_bd8e338fbd5f5cc8 = function(arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return addHeapObject(ret);
};
imports.wbg.__wbg_isArray_2ab64d95e09ea0ae = function(arg0) {
    const ret = Array.isArray(getObject(arg0));
    return ret;
};
imports.wbg.__wbg_length_cd7af8117672b8b8 = function(arg0) {
    const ret = getObject(arg0).length;
    return ret;
};
imports.wbg.__wbg_push_a5b05aedc7234f9f = function(arg0, arg1) {
    const ret = getObject(arg0).push(getObject(arg1));
    return ret;
};
imports.wbg.__wbg_new_28c511d9baebfa89 = function(arg0, arg1) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
    const ret = new Error(v0);
    return addHeapObject(ret);
};
imports.wbg.__wbg_newnoargs_e258087cd0daa0ea = function(arg0, arg1) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
    const ret = new Function(v0);
    return addHeapObject(ret);
};
imports.wbg.__wbg_call_27c0f87801dedf93 = function() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).call(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_call_b3ca7c6051f9bec1 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_next_196c84450b364254 = function() { return handleError(function (arg0) {
    const ret = getObject(arg0).next();
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_next_40fc327bfc8770e6 = function(arg0) {
    const ret = getObject(arg0).next;
    return addHeapObject(ret);
};
imports.wbg.__wbg_done_298b57d23c0fc80c = function(arg0) {
    const ret = getObject(arg0).done;
    return ret;
};
imports.wbg.__wbg_value_d93c65011f51a456 = function(arg0) {
    const ret = getObject(arg0).value;
    return addHeapObject(ret);
};
imports.wbg.__wbg_now_3014639a94423537 = function() {
    const ret = Date.now();
    return ret;
};
imports.wbg.__wbg_instanceof_Object_71ca3c0a59266746 = function(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Object;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};
imports.wbg.__wbg_getOwnPropertyDescriptor_fcb32c9a1f90b136 = function(arg0, arg1) {
    const ret = Object.getOwnPropertyDescriptor(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_is_010fdc0f4ab96916 = function(arg0, arg1) {
    const ret = Object.is(getObject(arg0), getObject(arg1));
    return ret;
};
imports.wbg.__wbg_new_72fb9a18b5ae2624 = function() {
    const ret = new Object();
    return addHeapObject(ret);
};
imports.wbg.__wbg_valueOf_a0b7c836f68a054b = function(arg0) {
    const ret = getObject(arg0).valueOf();
    return addHeapObject(ret);
};
imports.wbg.__wbg_exec_b9996525463e30df = function(arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    const ret = getObject(arg0).exec(v0);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};
imports.wbg.__wbg_new_3c970fa9da0c5794 = function(arg0, arg1, arg2, arg3) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
    var v1 = getCachedStringFromWasm0(arg2, arg3);
    const ret = new RegExp(v0, v1);
    return addHeapObject(ret);
};
imports.wbg.__wbg_iterator_2cee6dadfd956dfa = function() {
    const ret = Symbol.iterator;
    return addHeapObject(ret);
};
imports.wbg.__wbg_new_81740750da40724f = function(arg0, arg1) {
    try {
        var state0 = {a: arg0, b: arg1};
        var cb0 = (arg0, arg1) => {
            const a = state0.a;
            state0.a = 0;
            try {
                return __wbg_adapter_756(a, state0.b, arg0, arg1);
            } finally {
                state0.a = a;
            }
        };
        const ret = new Promise(cb0);
        return addHeapObject(ret);
    } finally {
        state0.a = state0.b = 0;
    }
};
imports.wbg.__wbg_resolve_b0083a7967828ec8 = function(arg0) {
    const ret = Promise.resolve(getObject(arg0));
    return addHeapObject(ret);
};
imports.wbg.__wbg_catch_0260e338d10f79ae = function(arg0, arg1) {
    const ret = getObject(arg0).catch(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_then_0c86a60e8fcfe9f6 = function(arg0, arg1) {
    const ret = getObject(arg0).then(getObject(arg1));
    return addHeapObject(ret);
};
imports.wbg.__wbg_then_a73caa9a87991566 = function(arg0, arg1, arg2) {
    const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
};
imports.wbg.__wbg_globalThis_d1e6af4856ba331b = function() { return handleError(function () {
    const ret = globalThis.globalThis;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_self_ce0dbfc45cf2f5be = function() { return handleError(function () {
    const ret = self.self;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_window_c6fb939a7f436783 = function() { return handleError(function () {
    const ret = window.window;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_global_207b558942527489 = function() { return handleError(function () {
    const ret = global.global;
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_new_63b92bc8671ed464 = function(arg0) {
    const ret = new Uint8Array(getObject(arg0));
    return addHeapObject(ret);
};
imports.wbg.__wbg_newwithbyteoffsetandlength_aa4a17c33a06e5cb = function(arg0, arg1, arg2) {
    const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};
imports.wbg.__wbg_buffer_dd7f74bc60f1faab = function(arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
};
imports.wbg.__wbg_length_c20a40f15020d68a = function(arg0) {
    const ret = getObject(arg0).length;
    return ret;
};
imports.wbg.__wbg_byteLength_58f7b4fab1919d44 = function(arg0) {
    const ret = getObject(arg0).byteLength;
    return ret;
};
imports.wbg.__wbg_byteOffset_81d60f7392524f62 = function(arg0) {
    const ret = getObject(arg0).byteOffset;
    return ret;
};
imports.wbg.__wbg_set_a47bac70306a19a7 = function(arg0, arg1, arg2) {
    getObject(arg0).set(getObject(arg1), arg2 >>> 0);
};
imports.wbg.__wbg_buffer_12d079cc21e14bdb = function(arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_is_string = function(arg0) {
    const ret = typeof(getObject(arg0)) === 'string';
    return ret;
};
imports.wbg.__wbg_get_e3c254076557e348 = function() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
}, arguments) };
imports.wbg.__wbg_set_1f9b04f170055d33 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
    return ret;
}, arguments) };
imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
    const ret = debugString(getObject(arg1));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};
imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
    takeObject(arg0);
};
imports.wbg.__wbindgen_throw = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};
imports.wbg.__wbindgen_memory = function() {
    const ret = wasm.memory;
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper2453 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 296, __wbg_adapter_40);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper2455 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 294, __wbg_adapter_43);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper4932 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 584, __wbg_adapter_46);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper5075 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 623, __wbg_adapter_49);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper7028 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 699, __wbg_adapter_52);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper7030 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 703, __wbg_adapter_55);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper7032 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 697, __wbg_adapter_58);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper7034 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 695, __wbg_adapter_61);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper7036 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 693, __wbg_adapter_64);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper7038 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 701, __wbg_adapter_67);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper8989 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1049, __wbg_adapter_70);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper15405 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1468, __wbg_adapter_73);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper18237 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1592, __wbg_adapter_76);
    return addHeapObject(ret);
};
imports.wbg.__wbindgen_closure_wrapper22876 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1601, __wbg_adapter_79);
    return addHeapObject(ret);
};

return imports;
}

function __wbg_init_memory(imports, maybe_memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedFloat64Memory0 = null;
    cachedInt32Memory0 = null;
    cachedUint32Memory0 = null;
    cachedUint8Memory0 = null;

    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(input) {
    if (wasm !== undefined) return wasm;

    if (typeof input === 'undefined') {
        input = new URL('kardashev-frontend-76225ad8f6ba7cc0_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await input, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync }
export default __wbg_init;
