/**
 * Pinia store for the graph editor.
 *
 * Holds the graph state (nodes, edges, selection) and provides
 * actions for adding, moving, connecting, and deleting nodes.
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { GraphNode, GraphEdge, GraphState, NodeKind } from '@/graph/types'
import { generateNodeId, generateEdgeId } from '@/graph/types'

export const useGraphStore = defineStore('graph', () => {
  // --- State ---
  const nodes = ref<GraphNode[]>([])
  const edges = ref<GraphEdge[]>([])
  const selectedNodeId = ref<string | null>(null)
  const selectedEdgeId = ref<string | null>(null)

  // --- Computed ---
  const selectedNode = computed(() =>
    nodes.value.find((n) => n.id === selectedNodeId.value) ?? null,
  )

  const selectedEdge = computed(() =>
    edges.value.find((e) => e.id === selectedEdgeId.value) ?? null,
  )

  const nodeCount = computed(() => nodes.value.length)
  const edgeCount = computed(() => edges.value.length)

  // --- Actions ---

  function addNode(kind: NodeKind, x: number, y: number, label?: string): GraphNode {
    const id = generateNodeId()
    const node: GraphNode = {
      id,
      kind,
      label: label ?? `${kind}-${nodes.value.length + 1}`,
      x,
      y,
    }
    nodes.value.push(node)
    selectedNodeId.value = id
    selectedEdgeId.value = null
    return node
  }

  function moveNode(id: string, x: number, y: number): void {
    const node = nodes.value.find((n) => n.id === id)
    if (node) {
      node.x = x
      node.y = y
    }
  }

  function removeNode(id: string): void {
    nodes.value = nodes.value.filter((n) => n.id !== id)
    edges.value = edges.value.filter((e) => e.from !== id && e.to !== id)
    if (selectedNodeId.value === id) selectedNodeId.value = null
  }

  function selectNode(id: string | null): void {
    selectedNodeId.value = id
    selectedEdgeId.value = null
  }

  function addEdge(from: string, to: string): GraphEdge | null {
    if (from === to) return null
    const exists = edges.value.some(
      (e) => (e.from === from && e.to === to) || (e.from === to && e.to === from),
    )
    if (exists) return null
    const edge: GraphEdge = {
      id: generateEdgeId(),
      from,
      to,
    }
    edges.value.push(edge)
    selectedEdgeId.value = edge.id
    selectedNodeId.value = null
    return edge
  }

  function removeEdge(id: string): void {
    edges.value = edges.value.filter((e) => e.id !== id)
    if (selectedEdgeId.value === id) selectedEdgeId.value = null
  }

  function selectEdge(id: string | null): void {
    selectedEdgeId.value = id
    selectedNodeId.value = null
  }

  function clearSelection(): void {
    selectedNodeId.value = null
    selectedEdgeId.value = null
  }

  function clear(): void {
    nodes.value = []
    edges.value = []
    selectedNodeId.value = null
    selectedEdgeId.value = null
  }

  function loadState(state: GraphState): void {
    nodes.value = [...state.nodes]
    edges.value = [...state.edges]
    selectedNodeId.value = state.selectedNodeId
    selectedEdgeId.value = state.selectedEdgeId
  }

  function getState(): GraphState {
    return {
      nodes: [...nodes.value],
      edges: [...edges.value],
      selectedNodeId: selectedNodeId.value,
      selectedEdgeId: selectedEdgeId.value,
    }
  }

  return {
    // State
    nodes,
    edges,
    selectedNodeId,
    selectedEdgeId,
    // Computed
    selectedNode,
    selectedEdge,
    nodeCount,
    edgeCount,
    // Actions
    addNode,
    moveNode,
    removeNode,
    selectNode,
    addEdge,
    removeEdge,
    selectEdge,
    clearSelection,
    clear,
    loadState,
    getState,
  }
})
