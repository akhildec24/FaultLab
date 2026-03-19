<script setup lang="ts">
import GraphEditor from '@/components/GraphEditor.vue'
import NodeInspector from '@/components/NodeInspector.vue'
import EdgeInspector from '@/components/EdgeInspector.vue'
import { useGraphStore } from '@/stores/graph'

const graph = useGraphStore()
</script>

<template>
  <div class="fl-width-full editor-view">
    <div class="editor-view__header">
      <div class="fl-container">
        <h1>Editor</h1>
        <p class="editor-view__subtitle">
          Build your topology by adding nodes and connecting them.
          Drag to reposition, scroll to zoom, click the amber handle to connect.
        </p>
      </div>
    </div>
    <div class="editor-view__body">
      <div class="editor-view__canvas">
        <GraphEditor />
      </div>
      <aside class="editor-view__inspector">
        <NodeInspector v-if="graph.selectedNodeId" />
        <EdgeInspector v-else-if="graph.selectedEdgeId" />
        <div class="inspector inspector--empty" v-else>
          <p class="inspector__empty-text">Select a node or connection to edit its properties</p>
        </div>
      </aside>
    </div>
  </div>
</template>

<style scoped>
.editor-view {
  display: flex;
  flex-direction: column;
  min-height: calc(100vh - 60px);
}

.editor-view__header {
  padding: var(--fl-space-3) 0;
  background: var(--fl-bg);
  border-bottom: 2px solid var(--fl-border);
}

.editor-view__header h1 {
  font-size: var(--fl-size-27);
}

.editor-view__subtitle {
  color: var(--fl-text-secondary);
  font-size: var(--fl-size-16);
  margin-top: var(--fl-space-1);
}

.editor-view__body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.editor-view__canvas {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.editor-view__inspector {
  width: 320px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.inspector--empty {
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--fl-bg);
  border-left: 2px solid var(--fl-border);
}

.inspector__empty-text {
  color: var(--fl-grey-3);
  font-size: var(--fl-size-16);
  text-align: center;
  padding: var(--fl-space-3);
}
</style>
