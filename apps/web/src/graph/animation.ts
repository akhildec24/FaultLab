/**
 * Animation types for rendering request movement on the graph.
 *
 * Events from the simulation worker are translated into animated
 * "particles" that travel along edges between nodes.
 */

/** A request particle moving between two nodes. */
export interface RequestParticle {
  requestId: number
  fromId: string
  toId: string
  /** Progress 0..1 along the edge. */
  progress: number
  /** Colour based on status. */
  status: 'transit' | 'processing' | 'success' | 'failed' | 'timeout' | 'queued'
}

/** A node flash effect (e.g. node failed, request arrived). */
export interface NodeFlash {
  nodeId: string
  color: string
  /** Remaining time in ms. */
  remaining: number
}

/** Sim event from the worker (matches Rust Event enum with tag). */
export interface SimEvent {
  time: number
  event: {
    type: string
    request_id?: number
    origin?: string
    node_id?: string
    success?: boolean
    queue_id?: string
    from?: string
    to?: string
    retry_count?: number
  }
}

/** Animation speed multiplier. */
export type SpeedMultiplier = 0.5 | 1 | 2 | 4

/** Particle colours by status. */
export const PARTICLE_COLORS: Record<RequestParticle['status'], string> = {
  transit: '#f59e0b',
  processing: '#6366f1',
  success: '#059669',
  failed: '#dc2626',
  timeout: '#b91c1c',
  queued: '#a1a1aa',
}

/** Flash duration in ms. */
export const FLASH_DURATION = 800
