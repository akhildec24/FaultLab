<script setup lang="ts">
/**
 * GraphEditor — interactive SVG canvas for building simulation topologies.
 *
 * Features:
 *   - Add nodes (client, service, database) via toolbar
 *   - Drag nodes to reposition
 *   - Click to select, Delete key to remove
 *   - Connect nodes: click a node's connect handle, then click target
 *   - Pan: drag on empty canvas
 *   - Zoom: mouse wheel or toolbar buttons
 */

import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useGraphStore } from '@/stores/graph'
import { useAnimationStore } from '@/stores/animation'
import {
  NODE_WIDTH,
  NODE_HEIGHT,
  NODE_COLORS,
  NODE_ICONS,
  type NodeKind,
} from '@/graph/types'
import { PARTICLE_COLORS } from '@/graph/animation'

const graph = useGraphStore()
const animation = useAnimationStore()

// --- View transform (pan/zoom) — stored in graph store ---
// Access via graph.view, graph.zoomIn, graph.zoomOut, graph.resetView

// --- SVG ref ---
const svgRef = ref<SVGSVGElement | null>(null)

// --- Viewport culling for large scenarios ---
const CULL_THRESHOLD = 50 // Only cull when more than this many nodes
const CULL_PADDING = 100 // Pixels of padding around viewport

const viewportBounds = computed(() => {
  const svg = svgRef.value
  if (!svg) return null
  const rect = svg.getBoundingClientRect()
  const zoom = graph.view.zoom
  const panX = graph.view.panX
  const panY = graph.view.panY
  // Convert screen viewport to graph coordinates
  const minX = (-panX - CULL_PADDING) / zoom
  const maxX = (rect.width - panX + CULL_PADDING) / zoom
  const minY = (-panY - CULL_PADDING) / zoom
  const maxY = (rect.height - panY + CULL_PADDING) / zoom
  return { minX, maxX, minY, maxY }
})

const shouldCull = computed(() => graph.nodes.length > CULL_THRESHOLD)

const visibleNodes = computed(() => {
  if (!shouldCull.value || !viewportBounds.value) return graph.nodes
  const { minX, maxX, minY, maxY } = viewportBounds.value
  return graph.nodes.filter((n) =>
    n.x + NODE_WIDTH >= minX && n.x <= maxX &&
    n.y + NODE_HEIGHT >= minY && n.y <= maxY,
  )
})

const visibleNodeIds = computed(() => {
  if (!shouldCull.value) return new Set(graph.nodes.map((n) => n.id))
  return new Set(visibleNodes.value.map((n) => n.id))
})

const visibleEdges = computed(() => {
  if (!shouldCull.value) return graph.edges
  const ids = visibleNodeIds.value
  return graph.edges.filter((e) => ids.has(e.from) && ids.has(e.to))
})

// Convert screen coordinates to graph coordinates
function screenToGraph(clientX: number, clientY: number): { x: number; y: number } {
  const svg = svgRef.value
  if (!svg) return { x: 0, y: 0 }
  const rect = svg.getBoundingClientRect()
  return {
    x: (clientX - rect.left - graph.view.panX) / graph.view.zoom,
    y: (clientY - rect.top - graph.view.panY) / graph.view.zoom,
  }
}

// --- Space + hand mode ---
const spaceMode = ref(false)

// --- Panning ---
const isPanning = ref(false)
let panStart = { x: 0, y: 0, panX: 0, panY: 0 }

function startPan(e: MouseEvent) {
  // In space mode, always pan regardless of target
  if (spaceMode.value) {
    isPanning.value = true
    panStart = {
      x: e.clientX,
      y: e.clientY,
      panX: graph.view.panX,
      panY: graph.view.panY,
    }
    return
  }
  if (e.target === svgRef.value || (e.target as Element).classList.contains('canvas-bg')) {
    isPanning.value = true
    panStart = {
      x: e.clientX,
      y: e.clientY,
      panX: graph.view.panX,
      panY: graph.view.panY,
    }
    graph.clearSelection()
  }
}

function onPan(e: MouseEvent) {
  if (!isPanning.value) return
  graph.view.panX = panStart.panX + (e.clientX - panStart.x)
  graph.view.panY = panStart.panY + (e.clientY - panStart.y)
}

function stopPan() {
  isPanning.value = false
}

// --- Node dragging ---
const draggedNode = ref<string | null>(null)
let dragOffset = { x: 0, y: 0 }

function startDragNode(e: MouseEvent, nodeId: string) {
  if (spaceMode.value) return // Don't drag nodes in hand mode
  e.stopPropagation()
  const node = graph.nodes.find((n) => n.id === nodeId)
  if (!node) return
  const pt = screenToGraph(e.clientX, e.clientY)
  dragOffset = { x: pt.x - node.x, y: pt.y - node.y }
  draggedNode.value = nodeId
  graph.selectNode(nodeId)
}

function onDragNode(e: MouseEvent) {
  if (!draggedNode.value) return
  const pt = screenToGraph(e.clientX, e.clientY)
  graph.moveNode(draggedNode.value, pt.x - dragOffset.x, pt.y - dragOffset.y)
}

function stopDragNode() {
  draggedNode.value = null
}

// --- Connection drawing ---
const connectMode = ref(false)
const connectFrom = ref<string | null>(null)
const mousePos = ref({ x: 0, y: 0 })

function startConnect(e: MouseEvent, nodeId: string) {
  e.stopPropagation()
  connectMode.value = true
  connectFrom.value = nodeId
  const pt = screenToGraph(e.clientX, e.clientY)
  mousePos.value = pt
}

function onConnectMove(e: MouseEvent) {
  if (!connectMode.value) return
  mousePos.value = screenToGraph(e.clientX, e.clientY)
}

function completeConnect(e: MouseEvent, nodeId: string) {
  if (!connectMode.value || !connectFrom.value) return
  e.stopPropagation()
  if (connectFrom.value !== nodeId) {
    graph.addEdge(connectFrom.value, nodeId)
  }
  connectMode.value = false
  connectFrom.value = null
}

function cancelConnect() {
  connectMode.value = false
  connectFrom.value = null
}

// --- Mouse move handler (combined) ---
function onMouseMove(e: MouseEvent) {
  onPan(e)
  onDragNode(e)
  onConnectMove(e)
}

function onMouseUp() {
  stopPan()
  stopDragNode()
}

// --- Keyboard ---
function onKeydown(e: KeyboardEvent) {
  if (e.code === 'Space' && !spaceMode.value) {
    e.preventDefault()
    spaceMode.value = true
    return
  }
  if (e.key === 'Delete' || e.key === 'Backspace') {
    if (graph.selectedNodeId) {
      graph.removeNode(graph.selectedNodeId)
    } else if (graph.selectedEdgeId) {
      graph.removeEdge(graph.selectedEdgeId)
    }
  }
  if (e.key === 'Escape') {
    cancelConnect()
    graph.clearSelection()
  }
}

function onKeyup(e: KeyboardEvent) {
  if (e.code === 'Space') {
    spaceMode.value = false
    isPanning.value = false
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown)
  window.addEventListener('keyup', onKeyup)
})
onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown)
  window.removeEventListener('keyup', onKeyup)
})

// --- Zoom on wheel ---
function onWheel(e: WheelEvent) {
  e.preventDefault()
  const delta = e.deltaY > 0 ? -0.1 : 0.1
  const newZoom = Math.max(0.3, Math.min(3, graph.view.zoom + delta))
  // Zoom toward cursor
  const svg = svgRef.value
  if (svg) {
    const rect = svg.getBoundingClientRect()
    const mx = e.clientX - rect.left
    const my = e.clientY - rect.top
    const ratio = newZoom / graph.view.zoom
    graph.view.panX = mx - (mx - graph.view.panX) * ratio
    graph.view.panY = my - (my - graph.view.panY) * ratio
  }
  graph.view.zoom = newZoom
}

// --- Add nodes ---
function addNode(kind: NodeKind) {
  const svg = svgRef.value
  if (!svg) return
  const rect = svg.getBoundingClientRect()
  const cx = (rect.width / 2 - graph.view.panX) / graph.view.zoom
  const cy = (rect.height / 2 - graph.view.panY) / graph.view.zoom
  // Offset slightly so nodes don't stack
  const offset = graph.nodeCount * 20
  graph.addNode(kind, cx + offset - NODE_WIDTH / 2, cy + offset - NODE_HEIGHT / 2)
}

// --- Edge path (bezier curve) ---
function edgePath(fromId: string, toId: string): string {
  const from = graph.nodes.find((n) => n.id === fromId)
  const to = graph.nodes.find((n) => n.id === toId)
  if (!from || !to) return ''
  const x1 = from.x + NODE_WIDTH
  const y1 = from.y + NODE_HEIGHT / 2
  const x2 = to.x
  const y2 = to.y + NODE_HEIGHT / 2
  const dx = Math.abs(x2 - x1) * 0.5
  return `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`
}

// --- Temp connection path ---
const tempEdgePath = computed(() => {
  if (!connectMode.value || !connectFrom.value) return ''
  const from = graph.nodes.find((n) => n.id === connectFrom.value)
  if (!from) return ''
  const x1 = from.x + NODE_WIDTH
  const y1 = from.y + NODE_HEIGHT / 2
  const x2 = mousePos.value.x
  const y2 = mousePos.value.y
  const dx = Math.abs(x2 - x1) * 0.5
  return `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`
})

// --- Node shape paths ---
// Returns SVG path data for different node shapes.
// All shapes fit within NODE_WIDTH x NODE_HEIGHT bounding box.
function nodeShape(kind: NodeKind): string {
  const w = NODE_WIDTH
  const h = NODE_HEIGHT
  const r = 6 // corner radius
  switch (kind) {
    case 'client':
      // Pill / fully rounded rect
      return `M ${h / 2} 0 L ${w - h / 2} 0 A ${h / 2} ${h / 2} 0 0 1 ${w - h / 2} ${h} L ${h / 2} ${h} A ${h / 2} ${h / 2} 0 0 1 ${h / 2} 0 Z`
    case 'database':
      // Cylinder: top ellipse + body + bottom ellipse
      const er = h * 0.15
      return `M 0 ${er} A ${w / 2} ${er} 0 0 0 ${w} ${er} L ${w} ${h - er} A ${w / 2} ${er} 0 0 1 0 ${h - er} Z`
    case 'queue':
      // Parallelogram (slanted rect)
      const skew = 12
      return `M ${skew} 0 L ${w} 0 L ${w - skew} ${h} L 0 ${h} Z`
    case 'cache':
      // Hexagon
      const hs = 14
      return `M ${hs} 0 L ${w - hs} 0 L ${w} ${h / 2} L ${w - hs} ${h} L ${hs} ${h} L 0 ${h / 2} Z`
    case 'external_api':
      // Cloud-ish shape (rounded bumps)
      return `M ${h * 0.3} ${h * 0.5} Q 0 ${h * 0.5} ${h * 0.2} ${h * 0.15} Q ${h * 0.4} 0 ${h * 0.7} ${h * 0.1} Q ${w * 0.3} 0 ${w * 0.45} ${h * 0.15} Q ${w} ${h * 0.1} ${w * 0.95} ${h * 0.4} Q ${w + 10} ${h * 0.7} ${w * 0.9} ${h * 0.85} Q ${w * 0.7} ${h} ${w * 0.5} ${h * 0.9} Q ${w * 0.2} ${h} ${h * 0.15} ${h * 0.85} Q -10 ${h * 0.7} ${h * 0.3} ${h * 0.5} Z`
    case 'service':
    default:
      // Standard rounded rectangle
      return `M ${r} 0 L ${w - r} 0 A ${r} ${r} 0 0 1 ${w} ${r} L ${w} ${h - r} A ${r} ${r} 0 0 1 ${w - r} ${h} L ${r} ${h} A ${r} ${r} 0 0 1 0 ${h - r} L 0 ${r} A ${r} ${r} 0 0 1 ${r} 0 Z`
  }
}

// --- Transform string ---
const transformStr = computed(() =>
  `translate(${graph.view.panX}, ${graph.view.panY}) scale(${graph.view.zoom})`,
)

// --- Animation helpers ---
function getNodeCenterX(nodeId: string): number {
  const node = graph.nodes.find((n) => n.id === nodeId)
  return node ? node.x + NODE_WIDTH / 2 : 0
}

function getNodeCenterY(nodeId: string): number {
  const node = graph.nodes.find((n) => n.id === nodeId)
  return node ? node.y + NODE_HEIGHT / 2 : 0
}

function getParticleX(p: { fromId: string; toId: string; progress: number }): number {
  const from = graph.nodes.find((n) => n.id === p.fromId)
  const to = graph.nodes.find((n) => n.id === p.toId)
  if (!from || !to) return 0
  const x1 = from.x + NODE_WIDTH
  const x2 = to.x
  return x1 + (x2 - x1) * p.progress
}

function getParticleY(p: { fromId: string; toId: string; progress: number }): number {
  const from = graph.nodes.find((n) => n.id === p.fromId)
  const to = graph.nodes.find((n) => n.id === p.toId)
  if (!from || !to) return 0
  const y1 = from.y + NODE_HEIGHT / 2
  const y2 = to.y + NODE_HEIGHT / 2
  return y1 + (y2 - y1) * p.progress
}

defineExpose({ addNode })
</script>

<template>
  <div class="graph-editor">
    <!-- SVG Canvas -->
    <svg
      ref="svgRef"
      class="graph-canvas"
      :class="{ 'graph-canvas--panning': isPanning, 'graph-canvas--connecting': connectMode, 'graph-canvas--hand': spaceMode }"
      @mousedown="startPan"
      @mousemove="onMouseMove"
      @mouseup="onMouseUp"
      @mouseleave="onMouseUp"
      @wheel.prevent="onWheel"
    >
      <!-- Background rect for pan/click -->
      <rect
        class="canvas-bg"
        x="0" y="0" width="100%" height="100%"
        :fill="'transparent'"
      />

      <g :transform="transformStr">
        <!-- Grid pattern -->
        <defs>
          <pattern id="grid" width="40" height="40" patternUnits="userSpaceOnUse">
            <circle cx="20" cy="20" r="1" fill="#d4d4d8" />
          </pattern>
        </defs>
        <rect x="-5000" y="-5000" width="10000" height="10000" fill="url(#grid)" />

        <!-- Edges -->
        <g class="edges">
          <path
            v-for="edge in visibleEdges"
            :key="edge.id"
            :d="edgePath(edge.from, edge.to)"
            :class="['edge', { 'edge--selected': edge.id === graph.selectedEdgeId }]"
            fill="none"
            @click.stop="graph.selectEdge(edge.id)"
          />
        </g>

        <!-- Temp edge while connecting -->
        <path
          v-if="tempEdgePath"
          :d="tempEdgePath"
          class="edge edge--temp"
          fill="none"
        />

        <!-- Nodes -->
        <g
          v-for="node in visibleNodes"
          :key="node.id"
          :class="['node', { 'node--selected': node.id === graph.selectedNodeId }]"
          :transform="`translate(${node.x}, ${node.y})`"
          @mousedown="connectMode ? completeConnect($event, node.id) : startDragNode($event, node.id)"
        >
          <!-- Node body (shape varies by kind) -->
          <path
            :d="nodeShape(node.kind)"
            :fill="NODE_COLORS[node.kind].fill"
            :stroke="NODE_COLORS[node.kind].stroke"
            :stroke-width="2"
          />
          <!-- Icon circle -->
          <circle
            :cx="20"
            :cy="NODE_HEIGHT / 2"
            r="12"
            :fill="NODE_COLORS[node.kind].stroke"
          />
          <text
            :x="20"
            :y="NODE_HEIGHT / 2 + 4"
            text-anchor="middle"
            fill="white"
            font-size="12"
            font-weight="700"
          >{{ NODE_ICONS[node.kind] }}</text>
          <!-- Label -->
          <text
            :x="40"
            :y="NODE_HEIGHT / 2 + 5"
            :fill="NODE_COLORS[node.kind].text"
            font-size="13"
            font-weight="600"
          >{{ node.label }}</text>
          <!-- Connect handle (right side) -->
          <circle
            :cx="NODE_WIDTH"
            :cy="NODE_HEIGHT / 2"
            r="6"
            class="node__handle"
            fill="#f59e0b"
            stroke="white"
            stroke-width="2"
            @mousedown.stop="startConnect($event, node.id)"
          />
        </g>

        <!-- Animation: node flashes -->
        <g class="flashes">
          <circle
            v-for="flash in animation.flashes"
            :key="flash.nodeId + '-' + flash.remaining"
            :cx="getNodeCenterX(flash.nodeId)"
            :cy="getNodeCenterY(flash.nodeId)"
            :r="NODE_WIDTH / 2 + 8"
            :fill="flash.color"
            :opacity="Math.min(flash.remaining / 800, 0.4)"
            pointer-events="none"
          />
        </g>

        <!-- Animation: request particles -->
        <g class="particles">
          <circle
            v-for="(p, i) in animation.particles"
            :key="p.requestId + '-' + i"
            :cx="getParticleX(p)"
            :cy="getParticleY(p)"
            r="6"
            :fill="PARTICLE_COLORS[p.status]"
            pointer-events="none"
          />
        </g>
      </g>
    </svg>

    <!-- Empty state -->
    <div v-if="graph.nodeCount === 0" class="graph-empty">
      <div class="graph-empty__icon">⚡</div>
      <h3 class="graph-empty__title">Start Building</h3>
      <p class="graph-empty__text">
        Add nodes from the toolbar above, or load a preset scenario from the dropdown.
      </p>
    </div>

    <!-- Connecting hint -->
    <div v-if="connectMode" class="graph-hint">
      Click a target node to connect · Esc to cancel
    </div>
  </div>
</template>

<style scoped>
.graph-editor {
  position: relative;
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.graph-canvas {
  flex: 1;
  width: 100%;
  min-height: 0;
  background: var(--fl-bg-alt);
  cursor: default;
  user-select: none;
}

.graph-canvas--panning {
  cursor: grabbing;
}

.graph-canvas--hand {
  cursor: grab;
}

.graph-canvas--hand.graph-canvas--panning {
  cursor: grabbing;
}

.graph-canvas--connecting {
  cursor: crosshair;
}

.canvas-bg {
  cursor: grab;
}

/* Edges */
.edge {
  stroke: var(--fl-grey-3);
  stroke-width: 2;
  cursor: pointer;
  transition: stroke 0.1s;
}

.edge:hover {
  stroke: var(--fl-amber);
  stroke-width: 3;
}

.edge--selected {
  stroke: var(--fl-amber);
  stroke-width: 3;
}

.edge--temp {
  stroke: var(--fl-amber);
  stroke-width: 2;
  stroke-dasharray: 6 4;
  pointer-events: none;
  opacity: 0.7;
}

/* Nodes */
.node {
  cursor: move;
}

.node--selected rect {
  stroke-width: 3;
  filter: drop-shadow(0 0 6px rgba(245, 158, 11, 0.5));
}

.node__handle {
  cursor: crosshair;
  opacity: 0;
  transition: opacity 0.15s;
}

.node:hover .node__handle {
  opacity: 1;
}

/* Empty state */
.graph-empty {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  pointer-events: none;
  text-align: center;
}

.graph-empty__icon {
  font-size: 48px;
  margin-bottom: var(--fl-space-2);
}

.graph-empty__title {
  font-size: var(--fl-size-24);
  font-weight: 700;
  color: var(--fl-slate);
  margin-bottom: var(--fl-space-1);
}

.graph-empty__text {
  font-size: var(--fl-size-16);
  color: var(--fl-grey-3);
  max-width: 320px;
}

/* Connecting hint */
.graph-hint {
  position: absolute;
  bottom: var(--fl-space-3);
  left: 50%;
  transform: translateX(-50%);
  background: var(--fl-slate);
  color: var(--fl-amber);
  padding: var(--fl-space-1) var(--fl-space-3);
  font-size: var(--fl-size-14);
  font-weight: 600;
  pointer-events: none;
}
</style>
