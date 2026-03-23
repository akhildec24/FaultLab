/**
 * Preset scenarios for quick loading into the graph editor.
 *
 * Each preset defines a complete topology with nodes, edges, and
 * simulation config — ready to run without manual editing.
 */

import type { GraphNode, GraphEdge } from './types'
import { generateNodeId, generateEdgeId } from './types'

export interface PresetScenario {
  id: string
  name: string
  description: string
  nodes: Omit<GraphNode, 'id'>[]
  edges: Omit<GraphEdge, 'id' | 'from' | 'to'>[]
  connections: [number, number][] // indices into nodes array
}

/** Overloaded database — client → service → database with low DB capacity. */
export const OVERLOADED_DATABASE: PresetScenario = {
  id: 'overloaded-database',
  name: 'Overloaded Database',
  description: 'A client sends traffic through a service to a database with low capacity. Watch requests queue and time out under load.',
  nodes: [
    {
      kind: 'client',
      label: 'Web Client',
      x: 80,
      y: 180,
      capacity: 100,
      latency_ms: 5,
      error_rate: 0,
      timeout_ms: 5000,
      queue_limit: null,
    },
    {
      kind: 'service',
      label: 'API Service',
      x: 340,
      y: 180,
      capacity: 50,
      latency_ms: 20,
      error_rate: 0,
      timeout_ms: 1000,
      queue_limit: 100,
    },
    {
      kind: 'database',
      label: 'Postgres DB',
      x: 600,
      y: 180,
      capacity: 10,
      latency_ms: 50,
      error_rate: 0,
      timeout_ms: 2000,
      queue_limit: 30,
    },
  ],
  edges: [
    { latency_ms: 10, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 15, packet_loss: 0, bandwidth_rps: 0 },
  ],
  connections: [[0, 1], [1, 2]],
}

/** Cascading failure — two services depend on a shared database. */
export const CASCADING_FAILURE: PresetScenario = {
  id: 'cascading-failure',
  name: 'Cascading Failure',
  description: 'Two services share a single database. When the database slows down, both services queue up and fail.',
  nodes: [
    {
      kind: 'client',
      label: 'Mobile Client',
      x: 60,
      y: 120,
      capacity: 100,
      latency_ms: 5,
      error_rate: 0,
      timeout_ms: 5000,
      queue_limit: null,
    },
    {
      kind: 'client',
      label: 'Web Client',
      x: 60,
      y: 280,
      capacity: 100,
      latency_ms: 5,
      error_rate: 0,
      timeout_ms: 5000,
      queue_limit: null,
    },
    {
      kind: 'service',
      label: 'Auth Service',
      x: 320,
      y: 120,
      capacity: 40,
      latency_ms: 30,
      error_rate: 0,
      timeout_ms: 1000,
      queue_limit: 80,
    },
    {
      kind: 'service',
      label: 'Order Service',
      x: 320,
      y: 280,
      capacity: 40,
      latency_ms: 25,
      error_rate: 0,
      timeout_ms: 1000,
      queue_limit: 80,
    },
    {
      kind: 'database',
      label: 'Shared DB',
      x: 580,
      y: 200,
      capacity: 8,
      latency_ms: 80,
      error_rate: 0.05,
      timeout_ms: 1500,
      queue_limit: 20,
    },
  ],
  edges: [
    { latency_ms: 10, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 10, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 15, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 15, packet_loss: 0, bandwidth_rps: 0 },
  ],
  connections: [[0, 2], [1, 3], [2, 4], [3, 4]],
}

/** Network partition — client connects to service with packet loss. */
export const NETWORK_PARTITION: PresetScenario = {
  id: 'network-partition',
  name: 'Network Partition',
  description: 'A client connects to a service through a lossy network. 20% packet loss causes retries and timeouts.',
  nodes: [
    {
      kind: 'client',
      label: 'API Client',
      x: 80,
      y: 200,
      capacity: 100,
      latency_ms: 5,
      error_rate: 0,
      timeout_ms: 3000,
      queue_limit: null,
    },
    {
      kind: 'service',
      label: 'Payment Service',
      x: 380,
      y: 200,
      capacity: 30,
      latency_ms: 40,
      error_rate: 0,
      timeout_ms: 800,
      queue_limit: 50,
    },
    {
      kind: 'database',
      label: 'Ledger DB',
      x: 660,
      y: 200,
      capacity: 25,
      latency_ms: 30,
      error_rate: 0,
      timeout_ms: 2000,
      queue_limit: 40,
    },
  ],
  edges: [
    { latency_ms: 50, packet_loss: 0.2, bandwidth_rps: 0 },
    { latency_ms: 10, packet_loss: 0, bandwidth_rps: 0 },
  ],
  connections: [[0, 1], [1, 2]],
}

export const PRESETS: PresetScenario[] = [
  OVERLOADED_DATABASE,
  CASCADING_FAILURE,
  NETWORK_PARTITION,
]

/** Convert a preset into graph nodes and edges with generated IDs. */
export function instantiatePreset(preset: PresetScenario): {
  nodes: GraphNode[]
  edges: GraphEdge[]
} {
  const nodes: GraphNode[] = preset.nodes.map((n) => ({
    ...n,
    id: generateNodeId(),
  }))

  const edges: GraphEdge[] = preset.connections.map(([fromIdx, toIdx], i) => ({
    id: generateEdgeId(),
    from: nodes[fromIdx].id,
    to: nodes[toIdx].id,
    ...preset.edges[i],
  }))

  return { nodes, edges }
}
