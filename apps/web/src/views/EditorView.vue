<script setup lang="ts">
import GraphEditor from '@/components/GraphEditor.vue'
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
    <div class="editor-view__canvas">
      <GraphEditor />
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

.editor-view__canvas {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
</style>
