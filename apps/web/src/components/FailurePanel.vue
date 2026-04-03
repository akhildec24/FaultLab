<script setup lang="ts">
/**
 * FailurePanel — UI for injecting failures mid-simulation.
 *
 * Allows crashing/recovering nodes, disconnecting/reconnecting links,
 * adding latency, packet loss, and reducing capacity.
 */

import { computed, ref } from 'vue'
import { useGraphStore } from '@/stores/graph'
import { useSimulationStore } from '@/stores/simulation'

const graph = useGraphStore()
const sim = useSimulationStore()

type FailureType =
  | 'crash'
  | 'recover'
  | 'add_latency'
  | 'add_packet_loss'
  | 'disconnect'
  | 'reduce_capacity'

const selectedFailure = ref<FailureType>('crash')
const selectedNodeId = ref<string>('')
const selectedEdgeFrom = ref<string>('')
const selectedEdgeTo = ref<string>('')
const latencyAmount = ref(100)
const packetLossRate = ref(0.1)
const newCapacity = ref(1)

const nodeOptions = computed(() =>
  graph.nodes.map((n) => ({ id: n.id, label: n.label })),
)

const edgeOptions = computed(() =>
  graph.edges.map((e) => ({
    id: `${e.from}->${e.to}`,
    from: e.from,
    to: e.to,
    label: `${nodeLabel(e.from)} → ${nodeLabel(e.to)}`,
  })),
)

function nodeLabel(id: string): string {
  return graph.nodes.find((n) => n.id === id)?.label ?? id
}

const failureTypes: { value: FailureType; label: string }[] = [
  { value: 'crash', label: 'Crash Node' },
  { value: 'recover', label: 'Recover Node' },
  { value: 'add_latency', label: 'Add Latency (Node)' },
  { value: 'add_packet_loss', label: 'Add Packet Loss (Link)' },
  { value: 'disconnect', label: 'Disconnect Link' },
  { value: 'reduce_capacity', label: 'Reduce Capacity' },
]

const isNodeFailure = computed(() =>
  ['crash', 'recover', 'add_latency', 'reduce_capacity'].includes(selectedFailure.value),
)

const isEdgeFailure = computed(() =>
  ['add_packet_loss', 'disconnect'].includes(selectedFailure.value),
)

const needsLatency = computed(() => selectedFailure.value === 'add_latency')
const needsPacketLoss = computed(() => selectedFailure.value === 'add_packet_loss')
const needsCapacity = computed(() => selectedFailure.value === 'reduce_capacity')

async function inject() {
  let json: string

  if (isNodeFailure.value) {
    if (!selectedNodeId.value) return
    switch (selectedFailure.value) {
      case 'crash':
        json = JSON.stringify({ type: 'crash', node_id: selectedNodeId.value })
        break
      case 'recover':
        json = JSON.stringify({ type: 'recover', node_id: selectedNodeId.value })
        break
      case 'add_latency':
        json = JSON.stringify({ type: 'add_latency', node_id: selectedNodeId.value, latency_ms: latencyAmount.value })
        break
      case 'reduce_capacity':
        json = JSON.stringify({ type: 'reduce_capacity', node_id: selectedNodeId.value, new_capacity: newCapacity.value })
        break
      default:
        return
    }
  } else {
    if (!selectedEdgeFrom.value || !selectedEdgeTo.value) return
    switch (selectedFailure.value) {
      case 'add_packet_loss':
        json = JSON.stringify({ type: 'add_packet_loss', from: selectedEdgeFrom.value, to: selectedEdgeTo.value, rate: packetLossRate.value })
        break
      case 'disconnect':
        json = JSON.stringify({ type: 'disconnect', from: selectedEdgeFrom.value, to: selectedEdgeTo.value })
        break
      default:
        return
    }
  }

  await sim.injectFailure(json)
}

const canInject = computed(() => {
  if (!sim.loaded) return false
  if (isNodeFailure.value) return !!selectedNodeId.value
  if (isEdgeFailure.value) return !!selectedEdgeFrom.value && !!selectedEdgeTo.value
  return false
})
</script>

<template>
  <div class="failure-panel">
    <span class="failure-panel__title">⚡ Failures</span>

    <select v-model="selectedFailure" class="failure-panel__select">
      <option v-for="ft in failureTypes" :key="ft.value" :value="ft.value">
        {{ ft.label }}
      </option>
    </select>

    <!-- Node selector -->
    <select v-if="isNodeFailure" v-model="selectedNodeId" class="failure-panel__select">
      <option value="" disabled>Node…</option>
      <option v-for="n in nodeOptions" :key="n.id" :value="n.id">
        {{ n.label }}
      </option>
    </select>

    <!-- Edge selector -->
    <select
      v-if="isEdgeFailure"
      :value="`${selectedEdgeFrom}->${selectedEdgeTo}`"
      @change="(e) => {
        const val = (e.target as HTMLSelectElement).value
        const [from, to] = val.split('->')
        selectedEdgeFrom = from
        selectedEdgeTo = to
      }"
      class="failure-panel__select"
    >
      <option value="" disabled>Link…</option>
      <option
        v-for="edge in edgeOptions"
        :key="edge.id"
        :value="edge.id"
      >
        {{ edge.label }}
      </option>
    </select>

    <!-- Latency input -->
    <input
      v-if="needsLatency"
      v-model.number="latencyAmount"
      type="number"
      min="1"
      max="10000"
      class="failure-panel__input"
      title="Latency (ms)"
    />

    <!-- Packet loss input -->
    <input
      v-if="needsPacketLoss"
      v-model.number="packetLossRate"
      type="number"
      min="0"
      max="1"
      step="0.05"
      class="failure-panel__input"
      title="Packet loss rate (0–1)"
    />

    <!-- Capacity input -->
    <input
      v-if="needsCapacity"
      v-model.number="newCapacity"
      type="number"
      min="0"
      max="1000"
      class="failure-panel__input"
      title="New capacity"
    />

    <button
      class="fl-button fl-button--warning failure-panel__btn"
      :disabled="!canInject"
      @click="inject"
    >
      Inject
    </button>
  </div>
</template>

<style scoped>
.failure-panel {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--fl-space-1);
  padding: var(--fl-space-1) var(--fl-space-2);
  background: var(--fl-slate-light);
  border-left: 2px solid var(--fl-amber);
  flex-shrink: 0;
}

.failure-panel__title {
  font-size: var(--fl-size-14);
  font-weight: 700;
  color: var(--fl-amber);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.failure-panel__select {
  font-family: var(--fl-font);
  font-size: var(--fl-size-14);
  padding: 4px 8px;
  background: var(--fl-slate);
  color: var(--fl-white);
  border: 1px solid var(--fl-slate);
  cursor: pointer;
  width: auto;
}

.failure-panel__select:focus {
  outline: 2px solid var(--fl-amber);
  outline-offset: -1px;
}

.failure-panel__input {
  font-family: var(--fl-font);
  font-size: var(--fl-size-14);
  padding: 4px 8px;
  background: var(--fl-slate);
  color: var(--fl-white);
  border: 1px solid var(--fl-slate);
  width: 70px;
}

.failure-panel__input:focus {
  outline: 2px solid var(--fl-amber);
  outline-offset: -1px;
}

.failure-panel__btn {
  font-size: var(--fl-size-14);
  padding: 4px 12px;
}

.failure-panel__btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
