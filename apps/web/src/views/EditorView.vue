<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import GraphEditor from '@/components/GraphEditor.vue'
import NodeInspector from '@/components/NodeInspector.vue'
import EdgeInspector from '@/components/EdgeInspector.vue'
import SimulationControls from '@/components/SimulationControls.vue'
import { useGraphStore } from '@/stores/graph'
import { useSimulationStore } from '@/stores/simulation'
import { useAnimationStore } from '@/stores/animation'
import type { NodeKind } from '@/graph/types'

const graph = useGraphStore()
const sim = useSimulationStore()
const animation = useAnimationStore()

const graphEditorRef = ref<InstanceType<typeof GraphEditor> | null>(null)

function addNode(kind: NodeKind): void {
  graphEditorRef.value?.addNode(kind)
}

// Wire simulation events → animation store
onMounted(() => {
  sim.onEvents((events) => animation.processEvents(events))
})

onUnmounted(() => {
  sim.onEvents(null)
  animation.stopLoop()
  animation.clear()
})
</script>

<template>
  <div class="editor-view">
    <div class="editor-view__header">
      <h1>Editor</h1>
      <p class="editor-view__subtitle">
        Build your topology by adding nodes and connecting them.
        Drag to reposition, scroll to zoom, click the amber handle to connect.
      </p>
    </div>
    <SimulationControls />
    <!-- Full-width graph toolbar -->
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
        <button class="fl-button fl-button--secondary graph-toolbar__btn" @click="graph.zoomOut">−</button>
        <span class="graph-toolbar__zoom">{{ Math.round(graph.view.zoom * 100) }}%</span>
        <button class="fl-button fl-button--secondary graph-toolbar__btn" @click="graph.zoomIn">+</button>
        <button class="fl-button fl-button--secondary graph-toolbar__btn" @click="graph.resetView">Reset</button>
      </div>
      <div class="graph-toolbar__group">
        <span class="graph-toolbar__info">
          {{ graph.nodeCount }} nodes · {{ graph.edgeCount }} edges
        </span>
      </div>
    </div>
    <!-- Main editor area: canvas + inspector side by side -->
    <div class="editor-view__body">
      <div class="editor-view__canvas">
        <GraphEditor ref="graphEditorRef" />
      </div>
      <aside class="editor-view__inspector">
        <NodeInspector v-if="graph.selectedNodeId" />
        <EdgeInspector v-else-if="graph.selectedEdgeId" />
        <div class="inspector-empty" v-else>
          <div class="inspector-empty__icon">?</div>
          <h3 class="inspector-empty__title">No Selection</h3>
          <p class="inspector-empty__text">
            Click a node or connection in the canvas to edit its properties.
          </p>
          <div class="inspector-empty__hints">
            <div class="inspector-empty__hint">
              <span class="inspector-empty__hint-key">Click</span>
              <span>Select a node</span>
            </div>
            <div class="inspector-empty__hint">
              <span class="inspector-empty__hint-key">Drag</span>
              <span>Move a node</span>
            </div>
            <div class="inspector-empty__hint">
              <span class="inspector-empty__hint-key">Scroll</span>
              <span>Zoom in/out</span>
            </div>
            <div class="inspector-empty__hint">
              <span class="inspector-empty__hint-key">○</span>
              <span>Drag amber handle to connect</span>
            </div>
          </div>
        </div>
      </aside>
    </div>
  </div>
</template>

<style scoped>
.editor-view {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.editor-view__header {
  padding: var(--fl-space-2) var(--fl-space-4);
  background: var(--fl-bg);
  border-bottom: 2px solid var(--fl-border);
  flex-shrink: 0;
}

.editor-view__header h1 {
  font-size: var(--fl-size-24);
}

.editor-view__subtitle {
  color: var(--fl-text-secondary);
  font-size: var(--fl-size-14);
  margin-top: var(--fl-space-1);
}

.editor-view__body {
  flex: 1;
  display: flex;
  min-height: 0;
  overflow: hidden;
}

.graph-toolbar {
  display: flex;
  align-items: center;
  gap: var(--fl-space-4);
  padding: var(--fl-space-2) var(--fl-space-3);
  background: var(--fl-slate);
  flex-shrink: 0;
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

.editor-view__canvas {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}

.editor-view__inspector {
  width: 320px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-left: 2px solid var(--fl-border);
  background: var(--fl-bg);
}

.inspector-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  min-height: 0;
  padding: var(--fl-space-4) var(--fl-space-3);
  text-align: center;
}

.inspector-empty__icon {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: var(--fl-slate);
  color: var(--fl-amber);
  font-size: var(--fl-size-24);
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: var(--fl-space-3);
}

.inspector-empty__title {
  font-size: var(--fl-size-19);
  font-weight: 700;
  color: var(--fl-text);
  margin-bottom: var(--fl-space-1);
}

.inspector-empty__text {
  font-size: var(--fl-size-14);
  color: var(--fl-grey-3);
  line-height: var(--fl-leading-normal);
  margin-bottom: var(--fl-space-4);
}

.inspector-empty__hints {
  display: flex;
  flex-direction: column;
  gap: var(--fl-space-2);
  width: 100%;
}

.inspector-empty__hint {
  display: flex;
  align-items: center;
  gap: var(--fl-space-2);
  font-size: var(--fl-size-14);
  color: var(--fl-text-secondary);
  text-align: left;
}

.inspector-empty__hint-key {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 44px;
  padding: 2px var(--fl-space-1);
  background: var(--fl-bg-alt);
  border: 1px solid var(--fl-border);
  border-radius: 4px;
  font-family: var(--fl-font-mono);
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--fl-slate);
  flex-shrink: 0;
}
</style>
