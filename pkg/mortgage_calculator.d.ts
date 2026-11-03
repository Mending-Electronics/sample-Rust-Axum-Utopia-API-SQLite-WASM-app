/* tslint:disable */
/* eslint-disable */

export function calculate_single_mortgage_json(json: string): string;

export function compare_mortgages_json(json: string): string;

export function handle_request(url: string, method: string, body?: string | null): Promise<any>;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly calculate_single_mortgage_json: (a: number, b: number) => [number, number, number, number];
    readonly compare_mortgages_json: (a: number, b: number) => [number, number, number, number];
    readonly handle_request: (a: number, b: number, c: number, d: number, e: number, f: number) => any;
    readonly wasm_bindgen_3cf8b40de5bb6d6b___convert__closures_____invoke___js_sys_310c67175162a3b9___Function_fn_wasm_bindgen_3cf8b40de5bb6d6b___JsValue_____wasm_bindgen_3cf8b40de5bb6d6b___sys__Undefined___js_sys_310c67175162a3b9___Function_fn_wasm_bindgen_3cf8b40de5bb6d6b___JsValue_____wasm_bindgen_3cf8b40de5bb6d6b___sys__Undefined_______true_: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen_3cf8b40de5bb6d6b___convert__closures_____invoke___wasm_bindgen_3cf8b40de5bb6d6b___JsValue__core_37ee0ce0c4cd57d5___result__Result_____wasm_bindgen_3cf8b40de5bb6d6b___JsError___true_: (a: number, b: number, c: any) => [number, number];
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_destroy_closure: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
