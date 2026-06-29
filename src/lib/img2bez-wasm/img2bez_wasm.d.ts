/* tslint:disable */
/* eslint-disable */

export function start(): void;

/**
 * Trace, then place deterministically from explicit placement intent, and
 * return `{ "glyph": <Glyph>, "report": <PlacementReport> }` as JSON. This is
 * the headless path for generated/padded rasters: with `fitSource: "ink"` the
 * ink box is fitted to `[fitYMin, fitYMax]` regardless of image margins.
 * Mirrors the CLI `--fit-source ink` exactly (same library call).
 */
export function tracePlaceToJson(image_bytes: Uint8Array, config_json: string): string;

/**
 * Trace PNG/JPEG/BMP image bytes to UFO GLIF XML. `config_json` is a JSON
 * object (see `Config`); pass `"{}"` for all defaults.
 */
export function traceToGlif(image_bytes: Uint8Array, config_json: string): string;

/**
 * Trace image bytes to a UFO-faithful JSON glyph: `{ name, unicodes, advance,
 * unitsPerEm, outline: { contours: [{ points: [{ x, y, type?, smooth? }] }] } }`.
 * The structured form web editors and agents consume directly (no XML parsing).
 */
export function traceToJson(image_bytes: Uint8Array, config_json: string): string;

/**
 * Trace image bytes to an SVG `<path>` `d` string (y-up; flip for a y-down
 * SVG viewport).
 */
export function traceToSvg(image_bytes: Uint8Array, config_json: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly start: () => void;
    readonly tracePlaceToJson: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly traceToGlif: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly traceToJson: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly traceToSvg: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
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
