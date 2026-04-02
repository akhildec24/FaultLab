<script setup lang="ts">
/**
 * PresetSelector — dropdown to load preset scenarios into the graph.
 *
 * Clears the current graph and loads the selected preset's nodes/edges.
 */

import { ref } from 'vue'
import { useGraphStore } from '@/stores/graph'
import { useSimulationStore } from '@/stores/simulation'
import { useAnimationStore } from '@/stores/animation'
import { PRESETS } from '@/graph/presets'

const graph = useGraphStore()
const sim = useSimulationStore()
const animation = useAnimationStore()

const selectedPreset = ref<string>('')
const showDescription = ref<string>('')

function loadPreset() {
  const preset = PRESETS.find((p) => p.id === selectedPreset.value)
  if (!preset) return

  // Reset simulation state
  animation.clear()
  sim.reset()

  // Load the preset into the graph
  graph.loadPreset(preset)
  showDescription.value = preset.description
}

function clearGraph() {
  animation.clear()
  sim.reset()
  graph.clear()
  selectedPreset.value = ''
  showDescription.value = ''
}
</script>

<template>
  <div class="preset-selector">
    <div class="preset-selector__controls">
      <select
        v-model="selectedPreset"
        class="preset-selector__select"
        @change="loadPreset"
      >
        <option value="" disabled>{{ PRESETS.length }} preset scenarios…</option>
        <option v-for="p in PRESETS" :key="p.id" :value="p.id">
          {{ p.name }}
        </option>
      </select>
      <button
        class="fl-button fl-button--secondary preset-selector__btn"
        @click="clearGraph"
      >
        Clear
      </button>
    </div>
    <p v-if="showDescription" class="preset-selector__description">
      {{ showDescription }}
    </p>
  </div>
</template>

<style scoped>
.preset-selector {
  display: flex;
  flex-direction: column;
  gap: var(--fl-space-1);
}

.preset-selector__controls {
  display: flex;
  gap: var(--fl-space-1);
  align-items: center;
}

.preset-selector__select {
  font-family: var(--fl-font);
  font-size: var(--fl-size-14);
  padding: var(--fl-space-1) var(--fl-space-2);
  background: var(--fl-slate-light);
  color: var(--fl-white);
  border: 1px solid var(--fl-slate-light);
  cursor: pointer;
  min-width: 200px;
}

.preset-selector__select:focus {
  outline: 2px solid var(--fl-amber);
  outline-offset: -1px;
}

.preset-selector__btn {
  font-size: var(--fl-size-14);
  padding: var(--fl-space-1) var(--fl-space-2);
  color: var(--fl-white);
  border-color: var(--fl-slate-light);
  background: transparent;
}

.preset-selector__btn:hover {
  background: var(--fl-slate-light);
  color: var(--fl-white);
}

.preset-selector__description {
  font-size: var(--fl-size-14);
  color: var(--fl-grey-3);
  line-height: 1.4;
}
</style>
