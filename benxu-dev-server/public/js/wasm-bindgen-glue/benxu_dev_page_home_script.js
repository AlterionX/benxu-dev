(function() {
    const __exports = {};
    let wasm;

    function __wbg_elem_binding0(arg0, arg1, arg2) {
        wasm.__wbg_function_table.get(19)(arg0, arg1, arg2);
    }

    const heap = new Array(32);

    heap.fill(undefined);

    heap.push(undefined, null, true, false);

    let heap_next = heap.length;

    function addHeapObject(obj) {
        if (heap_next === heap.length) heap.push(heap.length + 1);
        const idx = heap_next;
        heap_next = heap[idx];

        heap[idx] = obj;
        return idx;
    }
    function __wbg_elem_binding1(arg0, arg1, arg2) {
        wasm.__wbg_function_table.get(23)(arg0, arg1, addHeapObject(arg2));
    }
    /**
    * @param {number} idx
    */
    __exports.set_slide = function(idx) {
        wasm.set_slide(idx);
    };

    /**
    */
    __exports.init = function() {
        wasm.init();
    };

function getObject(idx) { return heap[idx]; }

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8');

let cachegetUint8Memory = null;
function getUint8Memory() {
    if (cachegetUint8Memory === null || cachegetUint8Memory.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory;
}

function getStringFromWasm(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory().subarray(ptr, ptr + len));
}

function handleError(e) {
    wasm.__wbindgen_exn_store(addHeapObject(e));
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let WASM_VECTOR_LEN = 0;

let cachedTextEncoder = new TextEncoder('utf-8');

let passStringToWasm;
if (typeof cachedTextEncoder.encodeInto === 'function') {
    passStringToWasm = function(arg) {


        let size = arg.length;
        let ptr = wasm.__wbindgen_malloc(size);
        let offset = 0;
        {
            const mem = getUint8Memory();
            for (; offset < arg.length; offset++) {
                const code = arg.charCodeAt(offset);
                if (code > 0x7F) break;
                mem[ptr + offset] = code;
            }
        }

        if (offset !== arg.length) {
            arg = arg.slice(offset);
            ptr = wasm.__wbindgen_realloc(ptr, size, size = offset + arg.length * 3);
            const view = getUint8Memory().subarray(ptr + offset, ptr + size);
            const ret = cachedTextEncoder.encodeInto(arg, view);

            offset += ret.written;
        }
        WASM_VECTOR_LEN = offset;
        return ptr;
    };
} else {
    passStringToWasm = function(arg) {


        let size = arg.length;
        let ptr = wasm.__wbindgen_malloc(size);
        let offset = 0;
        {
            const mem = getUint8Memory();
            for (; offset < arg.length; offset++) {
                const code = arg.charCodeAt(offset);
                if (code > 0x7F) break;
                mem[ptr + offset] = code;
            }
        }

        if (offset !== arg.length) {
            const buf = cachedTextEncoder.encode(arg.slice(offset));
            ptr = wasm.__wbindgen_realloc(ptr, size, size = offset + buf.length);
            getUint8Memory().set(buf, ptr + offset);
            offset += buf.length;
        }
        WASM_VECTOR_LEN = offset;
        return ptr;
    };
}

let cachegetInt32Memory = null;
function getInt32Memory() {
    if (cachegetInt32Memory === null || cachegetInt32Memory.buffer !== wasm.memory.buffer) {
        cachegetInt32Memory = new Int32Array(wasm.memory.buffer);
    }
    return cachegetInt32Memory;
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

function init(module) {

    let result;
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
        takeObject(arg0);
    };
    imports.wbg.__wbg_log_50e2d8db811e4f15 = function(arg0, arg1) {
        console.log(getStringFromWasm(arg0, arg1));
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
    imports.wbg.__widl_instanceof_Window = function(arg0) {
        const ret = getObject(arg0) instanceof Window;
        return ret;
    };
    imports.wbg.__widl_f_set_property_CSSStyleDeclaration = function(arg0, arg1, arg2, arg3, arg4) {
        try {
            getObject(arg0).setProperty(getStringFromWasm(arg1, arg2), getStringFromWasm(arg3, arg4));
        } catch (e) {
            handleError(e)
        }
    };
    imports.wbg.__widl_f_add_1_DOMTokenList = function(arg0, arg1, arg2) {
        try {
            getObject(arg0).add(getStringFromWasm(arg1, arg2));
        } catch (e) {
            handleError(e)
        }
    };
    imports.wbg.__widl_f_contains_DOMTokenList = function(arg0, arg1, arg2) {
        const ret = getObject(arg0).contains(getStringFromWasm(arg1, arg2));
        return ret;
    };
    imports.wbg.__widl_f_remove_1_DOMTokenList = function(arg0, arg1, arg2) {
        try {
            getObject(arg0).remove(getStringFromWasm(arg1, arg2));
        } catch (e) {
            handleError(e)
        }
    };
    imports.wbg.__widl_f_query_selector_Document = function(arg0, arg1, arg2) {
        try {
            const ret = getObject(arg0).querySelector(getStringFromWasm(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        } catch (e) {
            handleError(e)
        }
    };
    imports.wbg.__widl_f_query_selector_all_Document = function(arg0, arg1, arg2) {
        try {
            const ret = getObject(arg0).querySelectorAll(getStringFromWasm(arg1, arg2));
            return addHeapObject(ret);
        } catch (e) {
            handleError(e)
        }
    };
    imports.wbg.__widl_f_ready_state_Document = function(arg0, arg1) {
        const ret = getObject(arg1).readyState;
        const ret0 = passStringToWasm(ret);
        const ret1 = WASM_VECTOR_LEN;
        getInt32Memory()[arg0 / 4 + 0] = ret0;
        getInt32Memory()[arg0 / 4 + 1] = ret1;
    };
    imports.wbg.__widl_f_class_list_Element = function(arg0) {
        const ret = getObject(arg0).classList;
        return addHeapObject(ret);
    };
    imports.wbg.__widl_f_add_event_listener_with_callback_EventTarget = function(arg0, arg1, arg2, arg3) {
        try {
            getObject(arg0).addEventListener(getStringFromWasm(arg1, arg2), getObject(arg3));
        } catch (e) {
            handleError(e)
        }
    };
    imports.wbg.__widl_instanceof_HTMLElement = function(arg0) {
        const ret = getObject(arg0) instanceof HTMLElement;
        return ret;
    };
    imports.wbg.__widl_f_style_HTMLElement = function(arg0) {
        const ret = getObject(arg0).style;
        return addHeapObject(ret);
    };
    imports.wbg.__widl_f_offset_height_HTMLElement = function(arg0) {
        const ret = getObject(arg0).offsetHeight;
        return ret;
    };
    imports.wbg.__widl_f_item_NodeList = function(arg0, arg1) {
        const ret = getObject(arg0).item(arg1 >>> 0);
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__widl_f_length_NodeList = function(arg0) {
        const ret = getObject(arg0).length;
        return ret;
    };
    imports.wbg.__widl_f_document_Window = function(arg0) {
        const ret = getObject(arg0).document;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__widl_f_set_interval_with_callback_and_timeout_and_arguments_0_Window = function(arg0, arg1, arg2) {
        try {
            const ret = getObject(arg0).setInterval(getObject(arg1), arg2);
            return ret;
        } catch (e) {
            handleError(e)
        }
    };
    imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
        const ret = getObject(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_call_1fc553129cb17c3c = function(arg0, arg1) {
        try {
            const ret = getObject(arg0).call(getObject(arg1));
            return addHeapObject(ret);
        } catch (e) {
            handleError(e)
        }
    };
    imports.wbg.__wbg_newnoargs_368b05293a3f44de = function(arg0, arg1) {
        const ret = new Function(getStringFromWasm(arg0, arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_globalThis_8df2c73db5eac245 = function() {
        try {
            const ret = globalThis.globalThis;
            return addHeapObject(ret);
        } catch (e) {
            handleError(e)
        }
    };
    imports.wbg.__wbg_self_937dd9f384d2384a = function() {
        try {
            const ret = self.self;
            return addHeapObject(ret);
        } catch (e) {
            handleError(e)
        }
    };
    imports.wbg.__wbg_window_425d3fa09c43ece4 = function() {
        try {
            const ret = window.window;
            return addHeapObject(ret);
        } catch (e) {
            handleError(e)
        }
    };
    imports.wbg.__wbg_global_2c090b42ef2744b9 = function() {
        try {
            const ret = global.global;
            return addHeapObject(ret);
        } catch (e) {
            handleError(e)
        }
    };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        const ret = getObject(arg0) === undefined;
        return ret;
    };
    imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
        const ret = debugString(getObject(arg1));
        const ret0 = passStringToWasm(ret);
        const ret1 = WASM_VECTOR_LEN;
        getInt32Memory()[arg0 / 4 + 0] = ret0;
        getInt32Memory()[arg0 / 4 + 1] = ret1;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm(arg0, arg1));
    };
    imports.wbg.__wbindgen_rethrow = function(arg0) {
        throw takeObject(arg0);
    };
    imports.wbg.__wbindgen_closure_wrapper66 = function(arg0, arg1, arg2) {
        const state = { a: arg0, b: arg1, cnt: 1 };
        const real = (arg0) => {
            state.cnt++;
            try {
                return __wbg_elem_binding1(state.a, state.b, arg0);
            } finally {
                if (--state.cnt === 0) {
                    wasm.__wbg_function_table.get(20)(state.a, state.b);
                    state.a = 0;
                }
            }
        }
        ;
        real.original = state;
        const ret = real;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper64 = function(arg0, arg1, arg2) {
        const state = { a: arg0, b: arg1, cnt: 1 };
        const real = (arg0) => {
            state.cnt++;
            try {
                return __wbg_elem_binding0(state.a, state.b, arg0);
            } finally {
                if (--state.cnt === 0) {
                    wasm.__wbg_function_table.get(20)(state.a, state.b);
                    state.a = 0;
                }
            }
        }
        ;
        real.original = state;
        const ret = real;
        return addHeapObject(ret);
    };

    if (module instanceof URL || typeof module === 'string' || module instanceof Request) {

        const response = fetch(module);
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            result = WebAssembly.instantiateStreaming(response, imports)
            .catch(e => {
                console.warn("`WebAssembly.instantiateStreaming` failed. Assuming this is because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
                return response
                .then(r => r.arrayBuffer())
                .then(bytes => WebAssembly.instantiate(bytes, imports));
            });
        } else {
            result = response
            .then(r => r.arrayBuffer())
            .then(bytes => WebAssembly.instantiate(bytes, imports));
        }
    } else {

        result = WebAssembly.instantiate(module, imports)
        .then(result => {
            if (result instanceof WebAssembly.Instance) {
                return { instance: result, module };
            } else {
                return result;
            }
        });
    }
    return result.then(({instance, module}) => {
        wasm = instance.exports;
        init.__wbindgen_wasm_module = module;
        wasm.__wbindgen_start();
        return wasm;
    });
}

self.wasm_bindgen = Object.assign(init, __exports);

})();
