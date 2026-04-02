/**
 * Preset scenarios for quick loading into the graph editor.
 *
 * Each preset defines a complete topology with nodes, edges, and
 * simulation config — ready to run without manual editing.
 */

import type { GraphNode, GraphEdge } from './types'
import { generateNodeId, generateEdgeId, DEFAULT_NODE_CONFIG, DEFAULT_EDGE_CONFIG } from './types'

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
      retry_policy: { strategy: 'immediate', max_retries: 3, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
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
      retry_policy: { strategy: 'exponential', max_retries: 3, jitter: 0.2, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
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
      retry_policy: { strategy: 'fixed', max_retries: 1, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
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
      retry_policy: { strategy: 'immediate', max_retries: 3, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
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
      retry_policy: { strategy: 'immediate', max_retries: 3, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
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
      retry_policy: { strategy: 'exponential', max_retries: 3, jitter: 0.2, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
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
      retry_policy: { strategy: 'exponential', max_retries: 3, jitter: 0.2, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
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
      retry_policy: { strategy: 'fixed', max_retries: 1, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
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
      retry_policy: { strategy: 'immediate', max_retries: 3, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
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
      retry_policy: { strategy: 'exponential', max_retries: 3, jitter: 0.2, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
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
      retry_policy: { strategy: 'fixed', max_retries: 1, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
  ],
  edges: [
    { latency_ms: 50, packet_loss: 0.2, bandwidth_rps: 0 },
    { latency_ms: 10, packet_loss: 0, bandwidth_rps: 0 },
  ],
  connections: [[0, 1], [1, 2]],
}

/** Retry storm — aggressive retries on a failing service amplify load. */
export const RETRY_STORM: PresetScenario = {
  id: 'retry-storm',
  name: 'Retry Storm',
  description: 'A client retries aggressively against a service with 30% error rate. Immediate retries with no budget amplify load — watch retries skyrocket and the service collapse.',
  nodes: [
    {
      kind: 'client',
      label: 'Aggressive Client',
      x: 80,
      y: 180,
      capacity: 100,
      latency_ms: 5,
      error_rate: 0,
      timeout_ms: 2000,
      queue_limit: null,
      retry_policy: { strategy: 'immediate', max_retries: 10, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
    {
      kind: 'service',
      label: 'Flaky Service',
      x: 340,
      y: 180,
      capacity: 20,
      latency_ms: 50,
      error_rate: 0.3,
      timeout_ms: 500,
      queue_limit: 50,
      retry_policy: { strategy: 'immediate', max_retries: 10, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
    {
      kind: 'database',
      label: 'Backend DB',
      x: 600,
      y: 180,
      capacity: 15,
      latency_ms: 30,
      error_rate: 0,
      timeout_ms: 1000,
      queue_limit: 30,
      retry_policy: { strategy: 'fixed', max_retries: 1, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
  ],
  edges: [
    { latency_ms: 10, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 10, packet_loss: 0, bandwidth_rps: 0 },
  ],
  connections: [[0, 1], [1, 2]],
}

/** Queue overflow — low capacity service with small queue demonstrates load shedding. */
export const QUEUE_OVERFLOW: PresetScenario = {
  id: 'queue-overflow',
  name: 'Queue Overflow',
  description: 'A high-traffic client overwhelms a low-capacity service with a small queue. Watch requests get shedded and dequeued as capacity frees up.',
  nodes: [
    {
      kind: 'client',
      label: 'Burst Client',
      x: 80,
      y: 180,
      capacity: 100,
      latency_ms: 5,
      error_rate: 0,
      timeout_ms: 3000,
      queue_limit: null,
      retry_policy: { strategy: 'immediate', max_retries: 2, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
    {
      kind: 'service',
      label: 'Rate-Limited API',
      x: 340,
      y: 180,
      capacity: 5,
      latency_ms: 100,
      error_rate: 0,
      timeout_ms: 2000,
      queue_limit: 10,
      retry_policy: { strategy: 'exponential', max_retries: 2, jitter: 0.1, budget: null },
      shed_policy: 'reject',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
    {
      kind: 'database',
      label: 'Cache Store',
      x: 600,
      y: 180,
      capacity: 30,
      latency_ms: 10,
      error_rate: 0,
      timeout_ms: 500,
      queue_limit: 50,
      retry_policy: { strategy: 'fixed', max_retries: 1, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
  ],
  edges: [
    { latency_ms: 10, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 5, packet_loss: 0, bandwidth_rps: 0 },
  ],
  connections: [[0, 1], [1, 2]],
}

/** Cache & replication — cache layer with 60% hit rate, leader/replica DB with lag. */
export const CACHE_REPLICATION: PresetScenario = {
  id: 'cache-replication',
  name: 'Cache & Replication',
  description: 'A cache layer with 60% hit rate fronts a leader database with a replica. Watch cache hits skip the DB, and stale reads from the replica due to replication lag.',
  nodes: [
    {
      kind: 'client',
      label: 'Read Client',
      x: 80,
      y: 200,
      capacity: 100,
      latency_ms: 5,
      error_rate: 0,
      timeout_ms: 5000,
      queue_limit: null,
      retry_policy: { strategy: 'immediate', max_retries: 2, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
    {
      kind: 'service',
      label: 'Cache Layer',
      x: 340,
      y: 120,
      capacity: 80,
      latency_ms: 5,
      error_rate: 0,
      timeout_ms: 500,
      queue_limit: 200,
      retry_policy: { strategy: 'exponential', max_retries: 2, jitter: 0.1, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
    {
      kind: 'database',
      label: 'Leader DB',
      x: 600,
      y: 120,
      capacity: 30,
      latency_ms: 40,
      error_rate: 0,
      timeout_ms: 2000,
      queue_limit: 50,
      retry_policy: { strategy: 'fixed', max_retries: 1, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'leader',
      replication_lag_ms: 0,
    },
    {
      kind: 'database',
      label: 'Replica DB',
      x: 600,
      y: 300,
      capacity: 30,
      latency_ms: 40,
      error_rate: 0,
      timeout_ms: 2000,
      queue_limit: 50,
      retry_policy: { strategy: 'fixed', max_retries: 1, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'replica',
      replication_lag_ms: 300,
    },
  ],
  edges: [
    { latency_ms: 10, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 10, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 5, packet_loss: 0, bandwidth_rps: 0 },
  ],
  connections: [[0, 1], [1, 2], [2, 3]],
}

/** Replication delay — leader/replica with lag causes stale reads. */
export const REPLICATION_DELAY: PresetScenario = {
  id: 'replication-delay',
  name: 'Replication Delay',
  description: 'A leader database replicates writes to a read replica with 500ms lag. Read-heavy traffic hits the replica and sees stale data — watch consistency issues unfold.',
  nodes: [
    {
      kind: 'client',
      label: 'Read-Heavy Client',
      x: 80,
      y: 200,
      capacity: 100,
      latency_ms: 5,
      error_rate: 0,
      timeout_ms: 5000,
      queue_limit: null,
      retry_policy: { strategy: 'immediate', max_retries: 2, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
    {
      kind: 'service',
      label: 'Read Service',
      x: 340,
      y: 120,
      capacity: 60,
      latency_ms: 15,
      error_rate: 0,
      timeout_ms: 1000,
      queue_limit: 100,
      retry_policy: { strategy: 'exponential', max_retries: 2, jitter: 0.1, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
    {
      kind: 'service',
      label: 'Write Service',
      x: 340,
      y: 320,
      capacity: 30,
      latency_ms: 20,
      error_rate: 0,
      timeout_ms: 1000,
      queue_limit: 50,
      retry_policy: { strategy: 'exponential', max_retries: 2, jitter: 0.1, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
    {
      kind: 'database',
      label: 'Leader DB',
      x: 600,
      y: 220,
      capacity: 25,
      latency_ms: 30,
      error_rate: 0,
      timeout_ms: 2000,
      queue_limit: 40,
      retry_policy: { strategy: 'fixed', max_retries: 1, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'leader',
      replication_lag_ms: 0,
    },
    {
      kind: 'database',
      label: 'Replica DB',
      x: 600,
      y: 80,
      capacity: 40,
      latency_ms: 10,
      error_rate: 0,
      timeout_ms: 1000,
      queue_limit: 80,
      retry_policy: { strategy: 'fixed', max_retries: 1, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'replica',
      replication_lag_ms: 500,
    },
  ],
  edges: [
    { latency_ms: 10, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 10, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 5, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 5, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 5, packet_loss: 0, bandwidth_rps: 0 },
  ],
  connections: [[0, 1], [0, 2], [1, 4], [2, 3], [3, 4]],
}

/** Microservice mesh — multiple services calling each other in a chain. */
export const MICROSERVICE_MESH: PresetScenario = {
  id: 'microservice-mesh',
  name: 'Microservice Mesh',
  description: 'A client calls an API gateway, which fans out to three services. Each service has different capacity and latency. Watch how bottlenecked services cause cascading delays.',
  nodes: [
    {
      kind: 'client',
      label: 'Mobile App',
      x: 60,
      y: 200,
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
    {
      kind: 'service',
      label: 'API Gateway',
      x: 280,
      y: 200,
      capacity: 80,
      latency_ms: 10,
      error_rate: 0,
      timeout_ms: 2000,
      queue_limit: 200,
      retry_policy: { strategy: 'exponential', max_retries: 2, jitter: 0.15, budget: 50 },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
    {
      kind: 'service',
      label: 'User Service',
      x: 520,
      y: 80,
      capacity: 40,
      latency_ms: 25,
      error_rate: 0.02,
      timeout_ms: 1000,
      queue_limit: 60,
      retry_policy: { strategy: 'exponential', max_retries: 3, jitter: 0.2, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
    {
      kind: 'service',
      label: 'Order Service',
      x: 520,
      y: 200,
      capacity: 25,
      latency_ms: 40,
      error_rate: 0.05,
      timeout_ms: 1500,
      queue_limit: 40,
      retry_policy: { strategy: 'exponential', max_retries: 3, jitter: 0.2, budget: null },
      shed_policy: 'reject',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
    {
      kind: 'service',
      label: 'Inventory Service',
      x: 520,
      y: 320,
      capacity: 50,
      latency_ms: 15,
      error_rate: 0,
      timeout_ms: 800,
      queue_limit: 80,
      retry_policy: { strategy: 'exponential', max_retries: 2, jitter: 0.1, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
    {
      kind: 'database',
      label: 'Shared DB',
      x: 760,
      y: 200,
      capacity: 15,
      latency_ms: 60,
      error_rate: 0.01,
      timeout_ms: 2000,
      queue_limit: 25,
      retry_policy: { strategy: 'fixed', max_retries: 1, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
  ],
  edges: [
    { latency_ms: 10, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 15, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 15, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 15, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 20, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 20, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 20, packet_loss: 0, bandwidth_rps: 0 },
  ],
  connections: [[0, 1], [1, 2], [1, 3], [1, 4], [2, 5], [3, 5], [4, 5]],
}

/** Thundering herd — sudden traffic spike to a cold cache. */
export const THUNDERING_HERD: PresetScenario = {
  id: 'thundering-herd',
  name: 'Thundering Herd',
  description: 'A sudden traffic spike hits a service with a cold cache. All requests miss the cache and hit the database simultaneously. Watch the DB get overwhelmed until the cache warms up.',
  nodes: [
    {
      kind: 'client',
      label: 'Spike Traffic',
      x: 80,
      y: 180,
      capacity: 200,
      latency_ms: 5,
      error_rate: 0,
      timeout_ms: 3000,
      queue_limit: null,
      retry_policy: { strategy: 'immediate', max_retries: 5, jitter: 0.1, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
    {
      kind: 'service',
      label: 'Cache Service',
      x: 340,
      y: 180,
      capacity: 100,
      latency_ms: 8,
      error_rate: 0,
      timeout_ms: 500,
      queue_limit: 200,
      retry_policy: { strategy: 'exponential', max_retries: 2, jitter: 0.1, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
    {
      kind: 'database',
      label: 'Origin DB',
      x: 600,
      y: 180,
      capacity: 8,
      latency_ms: 80,
      error_rate: 0,
      timeout_ms: 3000,
      queue_limit: 15,
      retry_policy: { strategy: 'fixed', max_retries: 1, jitter: 0, budget: null },
      shed_policy: 'drop',
      replication_role: 'standalone',
      replication_lag_ms: 0,
    },
  ],
  edges: [
    { latency_ms: 10, packet_loss: 0, bandwidth_rps: 0 },
    { latency_ms: 30, packet_loss: 0, bandwidth_rps: 0 },
  ],
  connections: [[0, 1], [1, 2]],
}

export const PRESETS: PresetScenario[] = [
  OVERLOADED_DATABASE,
  CASCADING_FAILURE,
  NETWORK_PARTITION,
  RETRY_STORM,
  QUEUE_OVERFLOW,
  CACHE_REPLICATION,
  REPLICATION_DELAY,
  MICROSERVICE_MESH,
  THUNDERING_HERD,
]

/**
 * Generate a large scenario with a configurable number of nodes.
 * Produces a layered topology: clients → services → databases.
 */
export function generateLargeScenario(nodeCount: number): {
  nodes: GraphNode[]
  edges: GraphEdge[]
} {
  const clientCount = Math.floor(nodeCount * 0.4)
  const serviceCount = Math.floor(nodeCount * 0.4)
  const dbCount = nodeCount - clientCount - serviceCount

  const nodes: GraphNode[] = []
  const edges: GraphEdge[] = []

  // Layout in columns
  const colSpacing = 220
  const rowSpacing = 80

  // Clients
  for (let i = 0; i < clientCount; i++) {
    nodes.push({
      ...DEFAULT_NODE_CONFIG.client,
      id: generateNodeId(),
      label: `Client-${i + 1}`,
      x: 40,
      y: 40 + i * rowSpacing,
    })
  }

  // Services
  for (let i = 0; i < serviceCount; i++) {
    nodes.push({
      ...DEFAULT_NODE_CONFIG.service,
      id: generateNodeId(),
      label: `Service-${i + 1}`,
      x: 40 + colSpacing,
      y: 40 + i * rowSpacing,
    })
  }

  // Databases
  for (let i = 0; i < dbCount; i++) {
    nodes.push({
      ...DEFAULT_NODE_CONFIG.database,
      id: generateNodeId(),
      label: `DB-${i + 1}`,
      x: 40 + colSpacing * 2,
      y: 40 + i * rowSpacing,
    })
  }

  // Edges: each client connects to 1-2 services, each service to 1-2 DBs
  for (let i = 0; i < clientCount; i++) {
    const targetService = i % serviceCount
    edges.push({
      ...DEFAULT_EDGE_CONFIG,
      id: generateEdgeId(),
      from: nodes[i].id,
      to: nodes[clientCount + targetService].id,
      latency_ms: 5,
      packet_loss: 0.01,
    })
    if (serviceCount > 1 && i % 2 === 0) {
      const altService = (targetService + 1) % serviceCount
      edges.push({
        ...DEFAULT_EDGE_CONFIG,
        id: generateEdgeId(),
        from: nodes[i].id,
        to: nodes[clientCount + altService].id,
        latency_ms: 10,
        packet_loss: 0.02,
      })
    }
  }

  for (let i = 0; i < serviceCount; i++) {
    const targetDb = i % dbCount
    edges.push({
      ...DEFAULT_EDGE_CONFIG,
      id: generateEdgeId(),
      from: nodes[clientCount + i].id,
      to: nodes[clientCount + serviceCount + targetDb].id,
      latency_ms: 8,
      packet_loss: 0.005,
    })
  }

  return { nodes, edges }
}

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
