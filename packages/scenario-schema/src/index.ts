/// Scenario schema — TypeScript types matching the Rust `Scenario` struct.
/// These types are shared between the Vue app and the worker protocol.

export type ComponentKind =
  | 'client'
  | 'service'
  | 'queue'
  | 'cache'
  | 'database'
  | 'external_api'

export type NodeState = 'healthy' | 'degraded' | 'failed' | 'recovering'

export type RetryStrategy =
  | 'immediate'
  | { fixed: { delay_ms: number } }
  | { exponential: { base_ms: number; max_delay_ms: number } }

export interface RetryPolicy {
  strategy: RetryStrategy
  maxRetries: number
  jitter: number
  budget?: number
}

export interface NodeConfig {
  id: string
  kind: ComponentKind
  name: string
  capacity: number
  latencyMs: number
  errorRate: number
  timeoutMs: number
  queueLimit?: number
  cacheHitRate?: number
  retryPolicy?: RetryPolicy
}

export interface ConnectionConfig {
  from: string
  to: string
  latencyMs: number
  packetLoss: number
  bandwidthRps?: number
}

export interface TrafficConfig {
  startRps: number
  targetRps: number
  rampSeconds: number
}

export interface Scenario {
  name: string
  nodes: NodeConfig[]
  connections: ConnectionConfig[]
  traffic: TrafficConfig
  seed: number
}

/// The six initial component types with default values.
export const COMPONENT_DEFAULTS: Record<ComponentKind, Partial<NodeConfig>> = {
  client: { kind: 'client', capacity: 100, latencyMs: 5, errorRate: 0, timeoutMs: 5000 },
  service: { kind: 'service', capacity: 50, latencyMs: 20, errorRate: 0.01, timeoutMs: 1000 },
  queue: { kind: 'queue', capacity: 100, latencyMs: 1, errorRate: 0, timeoutMs: 5000, queueLimit: 100 },
  cache: { kind: 'cache', capacity: 200, latencyMs: 2, errorRate: 0, timeoutMs: 500, cacheHitRate: 0.8 },
  database: { kind: 'database', capacity: 80, latencyMs: 25, errorRate: 0.005, timeoutMs: 2000 },
  external_api: { kind: 'external_api', capacity: 30, latencyMs: 100, errorRate: 0.02, timeoutMs: 3000 },
}

export const COMPONENT_LABELS: Record<ComponentKind, string> = {
  client: 'Client',
  service: 'Service',
  queue: 'Queue',
  cache: 'Cache',
  database: 'Database',
  external_api: 'External API',
}

export const DEFAULT_RETRY_POLICY: RetryPolicy = {
  strategy: 'immediate',
  maxRetries: 3,
  jitter: 0,
}
