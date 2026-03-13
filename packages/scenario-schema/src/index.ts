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

export interface Node {
  id: string
  kind: ComponentKind
  name: string
  state: NodeState
  capacity: number
  latencyMs: number
  errorRate: number
  timeoutMs: number
  queueLimit?: number
  cacheHitRate?: number
}

export interface Connection {
  from: string
  to: string
  latencyMs: number
  packetLoss: number
}

export interface Scenario {
  name: string
  nodes: Node[]
  connections: Connection[]
  trafficStartRps: number
  trafficTargetRps: number
  trafficRampSeconds: number
  seed: number
}

/// The six initial component types with default values.
export const COMPONENT_DEFAULTS: Record<ComponentKind, Partial<Node>> = {
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
