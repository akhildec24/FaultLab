/**
 * Types for the graph editor.
 *
 * The editor works with its own internal representation that maps
 * to the simulation scenario's `NodeConfig` and `ConnectionConfig`.
 */

export type NodeKind = 'client' | 'service' | 'database'

export type RetryStrategyType = 'immediate' | 'fixed' | 'exponential'

export type SheddingPolicyType = 'drop' | 'reject' | 'backpressure'

export type ReplicationRoleType = 'standalone' | 'leader' | 'replica'

export interface RetryPolicy {
  strategy: RetryStrategyType
  max_retries: number
  jitter: number
  budget: number | null
}

export interface GraphNode {
  id: string
  kind: NodeKind
  label: string
  x: number
  y: number
  // Config properties (mirror Rust NodeConfig)
  capacity: number
  latency_ms: number
  error_rate: number
  timeout_ms: number
  queue_limit: number | null
  retry_policy: RetryPolicy
  shed_policy: SheddingPolicyType
  replication_role: ReplicationRoleType
  replication_lag_ms: number
}

export interface GraphEdge {
  id: string
  from: string
  to: string
  // Config properties (mirror Rust ConnectionConfig)
  latency_ms: number
  packet_loss: number
  bandwidth_rps: number
}

export interface GraphState {
  nodes: GraphNode[]
  edges: GraphEdge[]
  selectedNodeId: string | null
  selectedEdgeId: string | null
}

export interface ViewTransform {
  panX: number
  panY: number
  zoom: number
}

/** Default node dimensions for rendering. */
export const NODE_WIDTH = 140
export const NODE_HEIGHT = 60

/** Node colours per kind, using the FaultLab palette. */
export const NODE_COLORS: Record<NodeKind, { fill: string; stroke: string; text: string }> = {
  client: { fill: '#fef3c7', stroke: '#f59e0b', text: '#1a1a2e' },
  service: { fill: '#e0e7ff', stroke: '#6366f1', text: '#1a1a2e' },
  database: { fill: '#dcfce7', stroke: '#059669', text: '#1a1a2e' },
}

/** Node icons per kind (simple text glyphs for SVG). */
export const NODE_ICONS: Record<NodeKind, string> = {
  client: 'C',
  service: 'S',
  database: 'D',
}

/** Default config values per node kind. */
export const DEFAULT_NODE_CONFIG: Record<NodeKind, Omit<GraphNode, 'id' | 'x' | 'y'>> = {
  client: {
    kind: 'client',
    label: 'Client',
    capacity: 100,
    latency_ms: 5,
    error_rate: 0,
    timeout_ms: 5000,
    queue_limit: null,
    retry_policy: { strategy: 'immediate', max_retries: 3, jitter: 0, budget: null },
    shed_policy: 'drop',
    replication_role: 'standalone',
    replication_lag_ms: 0,
  },
  service: {
    kind: 'service',
    label: 'Service',
    capacity: 50,
    latency_ms: 20,
    error_rate: 0,
    timeout_ms: 1000,
    queue_limit: 100,
    retry_policy: { strategy: 'exponential', max_retries: 3, jitter: 0.2, budget: null },
    shed_policy: 'drop',
    replication_role: 'standalone',
    replication_lag_ms: 0,
  },
  database: {
    kind: 'database',
    label: 'Database',
    capacity: 20,
    latency_ms: 50,
    error_rate: 0,
    timeout_ms: 2000,
    queue_limit: 50,
    retry_policy: { strategy: 'fixed', max_retries: 1, jitter: 0, budget: null },
    shed_policy: 'drop',
    replication_role: 'standalone',
    replication_lag_ms: 0,
  },
}

/** Default config values for edges. */
export const DEFAULT_EDGE_CONFIG: Omit<GraphEdge, 'id' | 'from' | 'to'> = {
  latency_ms: 10,
  packet_loss: 0,
  bandwidth_rps: 0,
}

/** Generate a unique node ID. */
let nodeCounter = 0
export function generateNodeId(): string {
  nodeCounter++
  return `node-${Date.now()}-${nodeCounter}`
}

/** Generate a unique edge ID. */
let edgeCounter = 0
export function generateEdgeId(): string {
  edgeCounter++
  return `edge-${Date.now()}-${edgeCounter}`
}

/** Create a default graph state. */
export function createGraphState(): GraphState {
  return {
    nodes: [],
    edges: [],
    selectedNodeId: null,
    selectedEdgeId: null,
  }
}

/** Create a default view transform. */
export function createViewTransform(): ViewTransform {
  return { panX: 0, panY: 0, zoom: 1 }
}
