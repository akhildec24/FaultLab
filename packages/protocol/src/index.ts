/// Worker message protocol — typed messages between the main thread
/// and the Web Worker running the Rust/WASM simulation engine.

export type WorkerRequest =
  | { type: 'LOAD_SCENARIO'; id: string; payload: string }
  | { type: 'START_SIMULATION'; id: string }
  | { type: 'PAUSE_SIMULATION'; id: string }
  | { type: 'STEP_SIMULATION'; id: string }
  | { type: 'RESET_SIMULATION'; id: string }
  | { type: 'INJECT_FAILURE'; id: string; payload: FailureInjection }
  | { type: 'GET_METRICS'; id: string }
  | { type: 'GET_RECENT_EVENTS'; id: string; limit: number }

export type WorkerResponse =
  | { type: 'METRICS_BATCH'; id: string; payload: MetricsPayload }
  | { type: 'EVENT_BATCH'; id: string; payload: EventPayload[] }
  | { type: 'SIMULATION_COMPLETE'; id: string }
  | { type: 'SIMULATION_ERROR'; id: string; message: string }
  | { type: 'SCENARIO_LOADED'; id: string }
  | { type: 'ACK'; id: string }

export interface FailureInjection {
  nodeId: string
  failure: 'crash' | 'latency' | 'disconnect' | 'packet_loss' | 'slow'
  value?: number
}

export interface MetricsPayload {
  totalRequests: number
  successful: number
  failed: number
  timedOut: number
  retries: number
  dropped: number
  currentRps: number
  avgLatencyMs: number
  p50LatencyMs: number
  p95LatencyMs: number
  p99LatencyMs: number
  queueDepths: Record<string, number>
  nodeUtilisation: Record<string, number>
}

export interface EventPayload {
  time: number
  type: string
  requestId?: number
  nodeId?: string
  success?: boolean
}
