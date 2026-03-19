<script setup lang="ts">
/**
 * NodeInspector — edit properties of the selected graph node.
 *
 * Fields: label, kind, capacity, latency, error rate, timeout, queue limit.
 * Inline validation with error messages.
 */

import { computed } from 'vue'
import { useGraphStore } from '@/stores/graph'
import type { NodeKind } from '@/graph/types'

const graph = useGraphStore()

const node = computed(() => graph.selectedNode)

// --- Validation ---
const errors = computed<Record<string, string>>(() => {
  const e: Record<string, string> = {}
  if (!node.value) return e
  const n = node.value
  if (!n.label.trim()) e.label = 'Name is required'
  if (n.capacity < 1) e.capacity = 'Must be at least 1'
  if (n.latency_ms < 0) e.latency_ms = 'Cannot be negative'
  if (n.error_rate < 0 || n.error_rate > 1) e.error_rate = 'Must be between 0 and 1'
  if (n.timeout_ms < 1) e.timeout_ms = 'Must be at least 1'
  if (n.queue_limit !== null && n.queue_limit < 0) e.queue_limit = 'Cannot be negative'
  return e
})

const isValid = computed(() => Object.keys(errors.value).length === 0)

// --- Update helpers ---
function update(patch: Record<string, unknown>): void {
  if (!node.value) return
  graph.updateNode(node.value.id, patch)
}

function updateNumber(field: keyof typeof node.value, value: string): void {
  const num = parseInt(value, 10)
  update({ [field]: isNaN(num) ? 0 : num })
}

function updateFloat(field: keyof typeof node.value, value: string): void {
  const num = parseFloat(value)
  update({ [field]: isNaN(num) ? 0 : num })
}

function updateQueueLimit(value: string): void {
  if (value === '' || value === 'none') {
    update({ queue_limit: null })
  } else {
    const num = parseInt(value, 10)
    update({ queue_limit: isNaN(num) ? null : num })
  }
}

function deleteNode(): void {
  if (node.value) graph.removeNode(node.value.id)
}

const kindOptions: { value: NodeKind; label: string }[] = [
  { value: 'client', label: 'Client' },
  { value: 'service', label: 'Service' },
  { value: 'database', label: 'Database' },
]
</script>

<template>
  <div class="inspector" v-if="node">
    <div class="inspector__header">
      <h3 class="inspector__title">Node Properties</h3>
      <button class="fl-button fl-button--warning inspector__delete" @click="deleteNode">
        Delete
      </button>
    </div>

    <div class="inspector__body">
      <!-- Label -->
      <div class="inspector__field">
        <label class="inspector__label" for="node-label">Name</label>
        <input
          id="node-label"
          class="inspector__input"
          :class="{ 'inspector__input--error': errors.label }"
          type="text"
          :value="node.label"
          @input="update({ label: ($event.target as HTMLInputElement).value })"
        />
        <span class="inspector__error" v-if="errors.label">{{ errors.label }}</span>
        <span class="inspector__hint" v-else>Display name for this node</span>
      </div>

      <!-- Kind -->
      <div class="inspector__field">
        <label class="inspector__label" for="node-kind">Type</label>
        <select
          id="node-kind"
          class="inspector__input inspector__select"
          :value="node.kind"
          @change="update({ kind: ($event.target as HTMLSelectElement).value as NodeKind })"
        >
          <option v-for="opt in kindOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </select>
        <span class="inspector__hint">Component type determines routing behaviour</span>
      </div>

      <!-- Capacity -->
      <div class="inspector__field">
        <label class="inspector__label" for="node-capacity">Capacity</label>
        <input
          id="node-capacity"
          class="inspector__input"
          :class="{ 'inspector__input--error': errors.capacity }"
          type="number"
          min="1"
          :value="node.capacity"
          @input="updateNumber('capacity', ($event.target as HTMLInputElement).value)"
        />
        <span class="inspector__error" v-if="errors.capacity">{{ errors.capacity }}</span>
        <span class="inspector__hint" v-else>Max concurrent requests</span>
      </div>

      <!-- Latency -->
      <div class="inspector__field">
        <label class="inspector__label" for="node-latency">Processing latency (ms)</label>
        <input
          id="node-latency"
          class="inspector__input"
          :class="{ 'inspector__input--error': errors.latency_ms }"
          type="number"
          min="0"
          :value="node.latency_ms"
          @input="updateNumber('latency_ms', ($event.target as HTMLInputElement).value)"
        />
        <span class="inspector__error" v-if="errors.latency_ms">{{ errors.latency_ms }}</span>
        <span class="inspector__hint" v-else>Base processing time per request</span>
      </div>

      <!-- Error rate -->
      <div class="inspector__field">
        <label class="inspector__label" for="node-error">Error rate</label>
        <input
          id="node-error"
          class="inspector__input"
          :class="{ 'inspector__input--error': errors.error_rate }"
          type="number"
          min="0"
          max="1"
          step="0.01"
          :value="node.error_rate"
          @input="updateFloat('error_rate', ($event.target as HTMLInputElement).value)"
        />
        <span class="inspector__error" v-if="errors.error_rate">{{ errors.error_rate }}</span>
        <span class="inspector__hint" v-else>Probability of failure (0 = never, 1 = always)</span>
      </div>

      <!-- Timeout -->
      <div class="inspector__field">
        <label class="inspector__label" for="node-timeout">Timeout (ms)</label>
        <input
          id="node-timeout"
          class="inspector__input"
          :class="{ 'inspector__input--error': errors.timeout_ms }"
          type="number"
          min="1"
          :value="node.timeout_ms"
          @input="updateNumber('timeout_ms', ($event.target as HTMLInputElement).value)"
        />
        <span class="inspector__error" v-if="errors.timeout_ms">{{ errors.timeout_ms }}</span>
        <span class="inspector__hint" v-else>Request timeout threshold</span>
      </div>

      <!-- Queue limit -->
      <div class="inspector__field">
        <label class="inspector__label" for="node-queue">Queue limit</label>
        <input
          id="node-queue"
          class="inspector__input"
          :class="{ 'inspector__input--error': errors.queue_limit }"
          type="number"
          min="0"
          placeholder="none"
          :value="node.queue_limit ?? ''"
          @input="updateQueueLimit(($event.target as HTMLInputElement).value)"
        />
        <span class="inspector__error" v-if="errors.queue_limit">{{ errors.queue_limit }}</span>
        <span class="inspector__hint" v-else>Max queued requests (empty = no queue)</span>
      </div>
    </div>

    <div class="inspector__footer" v-if="!isValid">
      <span class="inspector__footer-error">Fix errors before running simulation</span>
    </div>
  </div>

  <div class="inspector inspector--empty" v-else>
    <p class="inspector__empty-text">Select a node to edit its properties</p>
  </div>
</template>

<style scoped>
.inspector {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--fl-bg);
  border-left: 2px solid var(--fl-border);
}

.inspector--empty {
  align-items: center;
  justify-content: center;
}

.inspector__empty-text {
  color: var(--fl-grey-3);
  font-size: var(--fl-size-16);
}

.inspector__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--fl-space-3);
  border-bottom: 2px solid var(--fl-border);
  background: var(--fl-slate);
}

.inspector__title {
  color: var(--fl-white);
  font-size: var(--fl-size-19);
  font-weight: 700;
}

.inspector__delete {
  font-size: var(--fl-size-14);
  padding: var(--fl-space-1) var(--fl-space-2);
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

.inspector__select {
  cursor: pointer;
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
