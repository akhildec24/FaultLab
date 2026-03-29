/**
 * Convert the visual graph into a simulation scenario JSON
 * that the Rust WASM engine can consume.
 *
 * Also provides validation with clear error messages.
 */

import type { GraphNode, GraphEdge } from './types'

// --- Scenario JSON types (mirror Rust structs) ---

export type RetryStrategyJson =
  | 'immediate'
  | { fixed: { delay_ms: number } }
  | { exponential: { base_ms: number; max_delay_ms: number } }

export interface ScenarioNode {
  id: string
  kind: string
  name: string
  capacity: number
  latency_ms: number
  error_rate: number
  timeout_ms: number
  queue_limit: number | null
  retry_policy: {
    strategy: RetryStrategyJson
    max_retries: number
    jitter: number
    budget?: number | null
  }
  shed_policy: { type: string }
  replication_role: string
  replication_lag_ms: number
}

export interface ScenarioConnection {
  from: string
  to: string
  latency_ms: number
  packet_loss: number
  bandwidth_rps: number
}

export interface ScenarioTraffic {
  start_rps: number
  target_rps: number
  ramp_seconds: number
}

export interface Scenario {
  name: string
  nodes: ScenarioNode[]
  connections: ScenarioConnection[]
  traffic: ScenarioTraffic
  seed: number
}

// --- Validation ---

export interface ValidationResult {
  valid: boolean
  errors: string[]
  warnings: string[]
}

export function validateGraph(
  nodes: GraphNode[],
  edges: GraphEdge[],
): ValidationResult {
  const errors: string[] = []
  const warnings: string[] = []

  if (nodes.length === 0) {
    errors.push('Add at least one node')
  }

  // Check for duplicate labels
  const labels = new Set<string>()
  for (const node of nodes) {
    if (!node.label.trim()) {
      errors.push(`Node "${node.id}" has no name`)
    }
    if (labels.has(node.label)) {
      errors.push(`Duplicate node name: "${node.label}"`)
    }
    labels.add(node.label)

    if (node.capacity < 1) {
      errors.push(`Node "${node.label}": capacity must be at least 1`)
    }
    if (node.error_rate < 0 || node.error_rate > 1) {
      errors.push(`Node "${node.label}": error rate must be between 0 and 1`)
    }
    if (node.timeout_ms < 1) {
      errors.push(`Node "${node.label}": timeout must be at least 1ms`)
    }
    if (node.queue_limit !== null && node.queue_limit < 0) {
      errors.push(`Node "${node.label}": queue limit cannot be negative`)
    }
  }

  // Check for at least one client
  const hasClient = nodes.some((n) => n.kind === 'client')
  if (nodes.length > 0 && !hasClient) {
    warnings.push('No client node — simulation will have no traffic source')
  }

  // Check for isolated nodes (no connections)
  if (nodes.length > 1) {
    for (const node of nodes) {
      const connected = edges.some(
        (e) => e.from === node.id || e.to === node.id,
      )
      if (!connected) {
        warnings.push(`Node "${node.label}" is not connected to anything`)
      }
    }
  }

  // Validate edges
  const nodeIds = new Set(nodes.map((n) => n.id))
  for (const edge of edges) {
    if (!nodeIds.has(edge.from)) {
      errors.push(`Connection references missing node: ${edge.from}`)
    }
    if (!nodeIds.has(edge.to)) {
      errors.push(`Connection references missing node: ${edge.to}`)
    }
    if (edge.latency_ms < 0) {
      errors.push('Connection latency cannot be negative')
    }
    if (edge.packet_loss < 0 || edge.packet_loss > 1) {
      errors.push('Connection packet loss must be between 0 and 1')
    }
    if (edge.bandwidth_rps < 0) {
      errors.push('Connection bandwidth cannot be negative')
    }
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings,
  }
}

// --- Conversion ---

const KIND_MAP: Record<string, string> = {
  client: 'client',
  service: 'service',
  database: 'database',
}

const DEFAULT_TRAFFIC: ScenarioTraffic = {
  start_rps: 5,
  target_rps: 10,
  ramp_seconds: 5,
}

const DEFAULT_SEED = 42

function retryStrategyToJson(strategy: string): RetryStrategyJson {
  switch (strategy) {
    case 'immediate':
      return 'immediate'
    case 'fixed':
      return { fixed: { delay_ms: 100 } }
    case 'exponential':
      return { exponential: { base_ms: 50, max_delay_ms: 5000 } }
    default:
      return 'immediate'
  }
}

export function graphToScenario(
  nodes: GraphNode[],
  edges: GraphEdge[],
  options?: {
    name?: string
    seed?: number
    traffic?: Partial<ScenarioTraffic>
  },
): Scenario {
  const scenarioNodes: ScenarioNode[] = nodes.map((n) => ({
    id: n.id,
    kind: KIND_MAP[n.kind] ?? 'service',
    name: n.label,
    capacity: n.capacity,
    latency_ms: n.latency_ms,
    error_rate: n.error_rate,
    timeout_ms: n.timeout_ms,
    queue_limit: n.queue_limit,
    retry_policy: {
      strategy: retryStrategyToJson(n.retry_policy.strategy),
      max_retries: n.retry_policy.max_retries,
      jitter: n.retry_policy.jitter,
      budget: n.retry_policy.budget,
    },
    shed_policy: { type: n.shed_policy },
    replication_role: n.replication_role,
    replication_lag_ms: n.replication_lag_ms,
  }))

  const scenarioConnections: ScenarioConnection[] = edges.map((e) => ({
    from: e.from,
    to: e.to,
    latency_ms: e.latency_ms,
    packet_loss: e.packet_loss,
    bandwidth_rps: e.bandwidth_rps,
  }))

  return {
    name: options?.name ?? 'Custom Scenario',
    nodes: scenarioNodes,
    connections: scenarioConnections,
    traffic: { ...DEFAULT_TRAFFIC, ...options?.traffic },
    seed: options?.seed ?? DEFAULT_SEED,
  }
}

export function graphToScenarioJson(
  nodes: GraphNode[],
  edges: GraphEdge[],
  options?: Parameters<typeof graphToScenario>[2],
): string {
  return JSON.stringify(graphToScenario(nodes, edges, options))
}
