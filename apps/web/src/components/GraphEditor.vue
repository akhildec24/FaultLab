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
  type ViewTransform,
} from '@/graph/types'
import { PARTICLE_COLORS } from '@/graph/animation'

const graph = useGraphStore()
const animation = useAnimationStore()

// --- View transform (pan/zoom) ---
const view = ref<ViewTransform>({ panX: 0, panY: 0, zoom: 1 })
const ZOOM_MIN = 0.3
const ZOOM_MAX = 3
const ZOOM_STEP = 0.1

function zoomIn() {
  view.value.zoom = Math.min(ZOOM_MAX, view.value.zoom + ZOOM_STEP)
}
function zoomOut() {
  view.value.zoom = Math.max(ZOOM_MIN, view.value.zoom - ZOOM_STEP)
}
function resetView() {
  view.value = { panX: 0, panY: 0, zoom: 1 }
}

// --- SVG ref ---
const svgRef = ref<SVGSVGElement | null>(null)

// Convert screen coordinates to graph coordinates
function screenToGraph(clientX: number, clientY: number): { x: number; y: number } {
  const svg = svgRef.value
  if (!svg) return { x: 0, y: 0 }
  const rect = svg.getBoundingClientRect()
  return {
    x: (clientX - rect.left - view.value.panX) / view.value.zoom,
    y: (clientY - rect.top - view.value.panY) / view.value.zoom,
  }
}

// --- Panning ---
const isPanning = ref(false)
let panStart = { x: 0, y: 0, panX: 0, panY: 0 }

function startPan(e: MouseEvent) {
  if (e.target === svgRef.value || (e.target as Element).classList.contains('canvas-bg')) {
    isPanning.value = true
    panStart = {
      x: e.clientX,
      y: e.clientY,
      panX: view.value.panX,
      panY: view.value.panY,
    }
    graph.clearSelection()
  }
}

function onPan(e: MouseEvent) {
  if (!isPanning.value) return
  view.value.panX = panStart.panX + (e.clientX - panStart.x)
  view.value.panY = panStart.panY + (e.clientY - panStart.y)
}

function stopPan() {
  isPanning.value = false
}

// --- Node dragging ---
const draggedNode = ref<string | null>(null)
let dragOffset = { x: 0, y: 0 }

function startDragNode(e: MouseEvent, nodeId: string) {
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

onMounted(() => {
  window.addEventListener('keydown', onKeydown)
})
onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown)
})

// --- Zoom on wheel ---
function onWheel(e: WheelEvent) {
  e.preventDefault()
  const delta = e.deltaY > 0 ? -ZOOM_STEP : ZOOM_STEP
  const newZoom = Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, view.value.zoom + delta))
  // Zoom toward cursor
  const svg = svgRef.value
  if (svg) {
    const rect = svg.getBoundingClientRect()
    const mx = e.clientX - rect.left
    const my = e.clientY - rect.top
    const ratio = newZoom / view.value.zoom
    view.value.panX = mx - (mx - view.value.panX) * ratio
    view.value.panY = my - (my - view.value.panY) * ratio
  }
  view.value.zoom = newZoom
}

// --- Add nodes ---
function addNode(kind: NodeKind) {
  const svg = svgRef.value
  if (!svg) return
  const rect = svg.getBoundingClientRect()
  const cx = (rect.width / 2 - view.value.panX) / view.value.zoom
  const cy = (rect.height / 2 - view.value.panY) / view.value.zoom
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

// --- Transform string ---
const transformStr = computed(() =>
  `translate(${view.value.panX}, ${view.value.panY}) scale(${view.value.zoom})`,
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
</script>

<template>
  <div class="graph-editor">
    <!-- Toolbar -->
    <div class="graph-toolbar">
      <div class="graph-toolbar__group">
        <button class="fl-button fl-button--secondary graph-toolbar__btn" @click="addNode('client')">
          + Client
        </button>
        <button class="fl-button fl-button--secondary graph-toolbar__btn" @click="addNode('service')">
          + Service
        </button>
        <button class="fl-button fl-button--secondary graph-toolbar__btn" @click="addNode('database')">
          + Database
        </button>
      </div>
      <div class="graph-toolbar__group">
        <button class="fl-button fl-button--secondary graph-toolbar__btn" @click="zoomOut">−</button>
        <span class="graph-toolbar__zoom">{{ Math.round(view.zoom * 100) }}%</span>
        <button class="fl-button fl-button--secondary graph-toolbar__btn" @click="zoomIn">+</button>
        <button class="fl-button fl-button--secondary graph-toolbar__btn" @click="resetView">Reset</button>
      </div>
      <div class="graph-toolbar__group">
        <span class="graph-toolbar__info">
          {{ graph.nodeCount }} nodes · {{ graph.edgeCount }} edges
        </span>
      </div>
    </div>

    <!-- SVG Canvas -->
    <svg
      ref="svgRef"
      class="graph-canvas"
      :class="{ 'graph-canvas--panning': isPanning, 'graph-canvas--connecting': connectMode }"
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
            v-for="edge in graph.edges"
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
          v-for="node in graph.nodes"
          :key="node.id"
          :class="['node', { 'node--selected': node.id === graph.selectedNodeId }]"
          :transform="`translate(${node.x}, ${node.y})`"
          @mousedown="connectMode ? completeConnect($event, node.id) : startDragNode($event, node.id)"
        >
          <!-- Node body -->
          <rect
            :width="NODE_WIDTH"
            :height="NODE_HEIGHT"
            :rx="6"
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
      <p>Add a node from the toolbar to get started</p>
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
  height: 100%;
  min-height: 500px;
}

.graph-toolbar {
  display: flex;
  align-items: center;
  gap: var(--fl-space-4);
  padding: var(--fl-space-2) var(--fl-space-3);
  background: var(--fl-slate);
  flex-wrap: wrap;
}

.graph-toolbar__group {
  display: flex;
  align-items: center;
  gap: var(--fl-space-1);
}

.graph-toolbar__btn {
  font-size: var(--fl-size-14);
  padding: var(--fl-space-1) var(--fl-space-2);
  color: var(--fl-white);
  border-color: var(--fl-slate-light);
  background: transparent;
}

.graph-toolbar__btn:hover {
  background: var(--fl-slate-light);
  color: var(--fl-white);
}

.graph-toolbar__zoom {
  color: var(--fl-grey-2);
  font-size: var(--fl-size-14);
  font-variant-numeric: tabular-nums;
  min-width: 48px;
  text-align: center;
}

.graph-toolbar__info {
  color: var(--fl-grey-2);
  font-size: var(--fl-size-14);
  margin-left: auto;
}

.graph-canvas {
  flex: 1;
  width: 100%;
  background: var(--fl-bg-alt);
  cursor: default;
  user-select: none;
}

.graph-canvas--panning {
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
  color: var(--fl-grey-3);
  font-size: var(--fl-size-19);
  pointer-events: none;
  text-align: center;
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
