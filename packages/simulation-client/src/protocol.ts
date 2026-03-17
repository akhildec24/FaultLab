/**
 * Typed message protocol for communication between the UI thread
 * and the simulation Web Worker.
 *
 * Every request has a unique `id` so the client can match responses.
 * Events (unsolicited notifications from the worker) use a separate
 * `SimulationEvent` type.
 */

// --- Request messages (UI → Worker) ---

export type WorkerRequest =
  | { id: number; type: 'LOAD_SCENARIO'; json: string }
  | { id: number; type: 'START' }
  | { id: number; type: 'PAUSE' }
  | { id: number; type: 'RESET' }
  | { id: number; type: 'STEP' }
  | { id: number; type: 'RUN'; maxSteps: number }
  | { id: number; type: 'GET_METRICS' }
  | { id: number; type: 'GET_STATE' }
  | { id: number; type: 'GET_RECENT_EVENTS' }
  | { id: number; type: 'GET_STATUS' }

// --- Response messages (Worker → UI) ---

export type WorkerResponse =
  | { id: number; type: 'OK'; payload: WorkerPayload }
  | { id: number; type: 'ERROR'; message: string }

export type WorkerPayload =
  | { kind: 'void' }
  | { kind: 'boolean'; value: boolean }
  | { kind: 'number'; value: number }
  | { kind: 'metrics'; json: string }
  | { kind: 'state'; json: string }
  | { kind: 'events'; json: string }
  | { kind: 'status'; running: boolean; currentTime: number; pendingEvents: number }

// --- Event messages (Worker → UI, unsolicited) ---

export type SimulationEvent =
  | { type: 'EVENTS'; events: unknown[] }
  | { type: 'ENGINE_STOPPED' }

// --- Union for all worker → UI messages ---

export type WorkerMessage = WorkerResponse | SimulationEvent
