<script setup lang="ts">
/**
 * EdgeInspector — edit properties of the selected graph edge.
 *
 * Fields: latency, packet loss, bandwidth.
 * Inline validation with error messages.
 */

import { computed } from 'vue'
import { useGraphStore } from '@/stores/graph'

const graph = useGraphStore()

const edge = computed(() => graph.selectedEdge)

const fromNode = computed(() =>
  graph.nodes.find((n) => n.id === edge.value?.from)?.label ?? '?',
)
const toNode = computed(() =>
  graph.nodes.find((n) => n.id === edge.value?.to)?.label ?? '?',
)

// --- Validation ---
const errors = computed<Record<string, string>>(() => {
  const e: Record<string, string> = {}
  if (!edge.value) return e
  const ed = edge.value
  if (ed.latency_ms < 0) e.latency_ms = 'Cannot be negative'
  if (ed.packet_loss < 0 || ed.packet_loss > 1) e.packet_loss = 'Must be between 0 and 1'
  if (ed.bandwidth_rps < 0) e.bandwidth_rps = 'Cannot be negative'
  return e
})

const isValid = computed(() => Object.keys(errors.value).length === 0)

// --- Update helpers ---
function update(patch: Record<string, unknown>): void {
  if (!edge.value) return
  graph.updateEdge(edge.value.id, patch)
}

function updateNumber(field: 'latency_ms' | 'bandwidth_rps', value: string): void {
  const num = parseInt(value, 10)
  update({ [field]: isNaN(num) ? 0 : num })
}

function updateFloat(field: 'packet_loss', value: string): void {
  const num = parseFloat(value)
  update({ [field]: isNaN(num) ? 0 : num })
}

function deleteEdge(): void {
  if (edge.value) graph.removeEdge(edge.value.id)
}
</script>

<template>
  <div class="inspector" v-if="edge">
    <div class="inspector__header">
      <div class="inspector__header-left">
        <span class="inspector__header-icon inspector__header-icon--edge">→</span>
        <h3 class="inspector__title">Connection</h3>
      </div>
      <button class="fl-button fl-button--warning inspector__delete" @click="deleteEdge">
        Delete
      </button>
    </div>

    <div class="inspector__route">
      <span class="inspector__route-node">{{ fromNode }}</span>
      <span class="inspector__route-arrow">→</span>
      <span class="inspector__route-node">{{ toNode }}</span>
    </div>

    <div class="inspector__body">
      <!-- Latency -->
      <div class="inspector__field">
        <label class="inspector__label" for="edge-latency">Latency (ms)</label>
        <input
          id="edge-latency"
          class="inspector__input"
          :class="{ 'inspector__input--error': errors.latency_ms }"
          type="number"
          min="0"
          :value="edge.latency_ms"
          @input="updateNumber('latency_ms', ($event.target as HTMLInputElement).value)"
        />
        <span class="inspector__error" v-if="errors.latency_ms">{{ errors.latency_ms }}</span>
        <span class="inspector__hint" v-else>Network transit time between nodes</span>
      </div>

      <!-- Packet loss -->
      <div class="inspector__field">
        <label class="inspector__label" for="edge-loss">Packet loss</label>
        <input
          id="edge-loss"
          class="inspector__input"
          :class="{ 'inspector__input--error': errors.packet_loss }"
          type="number"
          min="0"
          max="1"
          step="0.01"
          :value="edge.packet_loss"
          @input="updateFloat('packet_loss', ($event.target as HTMLInputElement).value)"
        />
        <span class="inspector__error" v-if="errors.packet_loss">{{ errors.packet_loss }}</span>
        <span class="inspector__hint" v-else>Probability of packet loss (0 = none, 1 = all)</span>
      </div>

      <!-- Bandwidth -->
      <div class="inspector__field">
        <label class="inspector__label" for="edge-bandwidth">Bandwidth (rps)</label>
        <input
          id="edge-bandwidth"
          class="inspector__input"
          :class="{ 'inspector__input--error': errors.bandwidth_rps }"
          type="number"
          min="0"
          :value="edge.bandwidth_rps"
          @input="updateNumber('bandwidth_rps', ($event.target as HTMLInputElement).value)"
        />
        <span class="inspector__error" v-if="errors.bandwidth_rps">{{ errors.bandwidth_rps }}</span>
        <span class="inspector__hint" v-else>Max requests per second (0 = unlimited)</span>
      </div>
    </div>

    <div class="inspector__footer" v-if="!isValid">
      <span class="inspector__footer-error">Fix errors before running simulation</span>
    </div>
  </div>
</template>

<style scoped>
.inspector {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--fl-bg);
}

.inspector__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--fl-space-3);
  border-bottom: 2px solid var(--fl-border);
  background: var(--fl-slate);
}

.inspector__header-left {
  display: flex;
  align-items: center;
  gap: var(--fl-space-2);
  min-width: 0;
}

.inspector__header-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  color: white;
  font-size: 13px;
  font-weight: 700;
  flex-shrink: 0;
}

.inspector__header-icon--edge {
  background: var(--fl-amber);
}

.inspector__title {
  color: var(--fl-white);
  font-size: var(--fl-size-19);
  font-weight: 700;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.inspector__delete {
  font-size: var(--fl-size-14);
  padding: var(--fl-space-1) var(--fl-space-2);
}

.inspector__route {
  display: flex;
  align-items: center;
  gap: var(--fl-space-2);
  padding: var(--fl-space-2) var(--fl-space-3);
  background: var(--fl-bg-alt);
  border-bottom: 2px solid var(--fl-border);
  font-size: var(--fl-size-16);
  font-weight: 600;
}

.inspector__route-node {
  color: var(--fl-text);
}

.inspector__route-arrow {
  color: var(--fl-amber);
  font-size: var(--fl-size-19);
}

.inspector__body {
  flex: 1;
  overflow-y: auto;
  padding: var(--fl-space-3);
}

.inspector__field {
  margin-bottom: var(--fl-space-3);
}

.inspector__label {
  display: block;
  font-size: var(--fl-size-14);
  font-weight: 600;
  color: var(--fl-text);
  margin-bottom: var(--fl-space-1);
}

.inspector__input {
  width: 100%;
  padding: var(--fl-space-1) var(--fl-space-2);
  border: 2px solid var(--fl-border);
  font-family: var(--fl-font);
  font-size: var(--fl-size-16);
  background: var(--fl-white);
  color: var(--fl-text);
}

.inspector__input:focus {
  outline: none;
  border-color: var(--fl-amber);
}

.inspector__input--error {
  border-color: var(--fl-red);
}

.inspector__error {
  display: block;
  font-size: var(--fl-size-14);
  color: var(--fl-red);
  margin-top: var(--fl-space-1);
}

.inspector__hint {
  display: block;
  font-size: var(--fl-size-14);
  color: var(--fl-grey-3);
  margin-top: var(--fl-space-1);
}

.inspector__footer {
  padding: var(--fl-space-2) var(--fl-space-3);
  border-top: 2px solid var(--fl-red);
  background: var(--fl-red-light);
}

.inspector__footer-error {
  font-size: var(--fl-size-14);
  color: var(--fl-red-hover);
  font-weight: 600;
}
</style>
