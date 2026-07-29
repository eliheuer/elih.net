/* tslint:disable */
/* eslint-disable */

export class NtfFont {
    free(): void;
    [Symbol.dispose](): void;
    alphabet(): string;
    max_elong(): number;
    n_params(): number;
    /**
     * Parse a .ntf font (a serialized neural model) from bytes.
     */
    constructor(bytes: Uint8Array);
    /**
     * Shape `text` at elongation `elong` ∈ [0, max_elong].
     * Returns JSON: { width, grid_h, baseline, path, glyphs } where
     * `path` is ONE SVG path for the whole line (connected letters
     * are one continuous contour) in grid units (y-down), and
     * `glyphs` is cluster metadata [{ch, form, x, advance}].
     */
    shape(text: string, elong: number, dir: string): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_ntffont_free: (a: number, b: number) => void;
    readonly ntffont_alphabet: (a: number) => [number, number];
    readonly ntffont_max_elong: (a: number) => number;
    readonly ntffont_n_params: (a: number) => number;
    readonly ntffont_new: (a: number, b: number) => [number, number, number];
    readonly ntffont_shape: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
