<script setup lang="ts">
import { ref } from 'vue'

const placeholder = ref('The visual graph editor arrives on Day 10.')
</script>

<template>
  <div class="fl-container fl-section">
    <h1>Editor</h1>
    <p class="editor__note">{{ placeholder }}</p>
    <div class="editor__placeholder">
      <div class="editor__placeholder-icon" aria-hidden="true">◇</div>
      <p>The interactive canvas will appear here once the graph editor is
      built (Day 10 of the build plan).</p>
      <p>For now, the simulation engine is being developed in Rust.</p>
    </div>
  </div>
</template>

<style scoped>
.editor__note {
  color: var(--fl-grey-4);
  font-size: var(--fl-size-19);
  margin-bottom: var(--fl-space-5);
}

.editor__placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: var(--fl-space-7) var(--fl-space-4);
  background: var(--fl-bg-alt);
  border: 2px dashed var(--fl-border);
}

.editor__placeholder-icon {
  font-size: var(--fl-size-48);
  color: var(--fl-grey-2);
  margin-bottom: var(--fl-space-4);
}

.editor__placeholder p {
  color: var(--fl-grey-4);
  max-width: 400px;
  margin-bottom: var(--fl-space-2);
}
</style>
