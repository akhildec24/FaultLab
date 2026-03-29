/* tslint:disable */
/* eslint-disable */

export class Simulation {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Current virtual time in milliseconds.
     */
    current_time(): bigint;
    /**
     * Current metrics as a JSON string.
     */
    get_metrics(): string;
    /**
     * Drain and return recent events as a JSON array string.
     * Each event is `{"time": <ms>, "event": {...}}`.
     */
    get_recent_events(): string;
    /**
     * Full simulation state as a JSON string (requests, nodes, network, metrics).
     */
    get_state(): string;
    /**
     * Inject a failure mid-simulation. Takes a JSON string representing
     * a `FailureInjection` enum variant.
     */
    inject_failure(json: string): void;
    /**
     * Whether the engine is currently running.
     */
    is_running(): boolean;
    /**
     * Load a scenario from a JSON string.
     */
    load_scenario(json: string): void;
    /**
     * Create a new simulation with an empty scenario.
     */
    constructor();
    /**
     * Pause the simulation.
     */
    pause(): void;
    /**
     * Number of pending events in the scheduler queue.
     */
    pending_events(): number;
    /**
     * Reset the simulation to its initial state.
     */
    reset(): void;
    /**
     * Process up to `max_steps` events. Returns the number of steps executed.
     */
    run(max_steps: number): number;
    /**
     * Start the simulation (schedules traffic if not already scheduled).
     */
    start(): void;
    /**
     * Process one event. Returns `true` if an event was processed.
     */
    step(): boolean;
}

/**
 * Initialise the panic hook for better error messages in the browser.
 */
export function init(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_simulation_free: (a: number, b: number) => void;
    readonly simulation_current_time: (a: number) => bigint;
    readonly simulation_get_metrics: (a: number) => [number, number];
    readonly simulation_get_recent_events: (a: number) => [number, number];
    readonly simulation_get_state: (a: number) => [number, number];
    readonly simulation_inject_failure: (a: number, b: number, c: number) => [number, number];
    readonly simulation_is_running: (a: number) => number;
    readonly simulation_load_scenario: (a: number, b: number, c: number) => [number, number];
    readonly simulation_new: () => number;
    readonly simulation_pause: (a: number) => void;
    readonly simulation_pending_events: (a: number) => number;
    readonly simulation_reset: (a: number) => void;
    readonly simulation_run: (a: number, b: number) => number;
    readonly simulation_start: (a: number) => void;
    readonly simulation_step: (a: number) => number;
    readonly init: () => void;
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
