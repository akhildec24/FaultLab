/**
 * Types for the graph editor.
 *
 * The editor works with its own internal representation that maps
 * to the simulation scenario's `NodeConfig` and `ConnectionConfig`.
 */

export type NodeKind = 'client' | 'service' | 'database'

export interface GraphNode {
  id: string
  kind: NodeKind
  label: string
  x: number
  y: number
}

export interface GraphEdge {
  id: string
  from: string
  to: string
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
