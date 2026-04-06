<script setup lang="ts">
/**
 * EventTimeline — searchable, filterable event log with timestamps.
 *
 * Displays simulation events as they arrive from the engine, with
 * filters by event type, component (node ID), and request ID.
 */

import { computed, ref } from 'vue'
import { useSimulationStore } from '@/stores/simulation'

const sim = useSimulationStore()

// --- Filters ---
const searchText = ref('')
const filterType = ref<string>('all')
const filterNodeId = ref<string>('all')
const filterRequestId = ref<string>('')

// --- Event type options (derived from actual events) ---
const eventTypes = computed(() => {
  const types = new Set<string>()
  for (const raw of sim.eventLog) {
    const evt = raw as { event?: { type?: string } }
    if (evt?.event?.type) types.add(evt.event.type)
  }
  return Array.from(types).sort()
})

// --- Node IDs from events (for component filter) ---
const nodeIds = computed(() => {
  const ids = new Set<string>()
  for (const raw of sim.eventLog) {
    const evt = raw as { event?: { node_id?: string; from?: string; to?: string; origin?: string } }
    const e = evt?.event
    if (e) {
      if (e.node_id) ids.add(e.node_id)
      if (e.from) ids.add(e.from)
      if (e.to) ids.add(e.to)
      if (e.origin) ids.add(e.origin)
    }
  }
  return Array.from(ids).sort()
})

// --- Filtered events (most recent first, limited display) ---
const DISPLAY_LIMIT = 200

const filteredEvents = computed(() => {
  let result = sim.eventLog as Array<{
    time: number
    event: {
      type: string
      request_id?: number
      node_id?: string
      origin?: string
      from?: string
      to?: string
      success?: boolean
      queue_id?: string
      retry_count?: number
      policy?: string
    }
  }>

  // Filter by event type
  if (filterType.value !== 'all') {
    result = result.filter((e) => e.event.type === filterType.value)
  }

  // Filter by node ID
  if (filterNodeId.value !== 'all') {
    const nid = filterNodeId.value
    result = result.filter((e) => {
      const ev = e.event
      return ev.node_id === nid || ev.from === nid || ev.to === nid || ev.origin === nid
    })
  }

  // Filter by request ID
  if (filterRequestId.value.trim()) {
    const rid = filterRequestId.value.trim()
    result = result.filter((e) => {
      if (e.event.request_id === undefined) return false
      return String(e.event.request_id) === rid
    })
  }

  // Filter by search text (matches type, node_id, etc.)
  if (searchText.value.trim()) {
    const q = searchText.value.toLowerCase()
    result = result.filter((e) => {
      const ev = e.event
      return (
        ev.type.toLowerCase().includes(q) ||
        (ev.node_id && ev.node_id.toLowerCase().includes(q)) ||
        (ev.from && ev.from.toLowerCase().includes(q)) ||
        (ev.to && ev.to.toLowerCase().includes(q)) ||
        (ev.origin && ev.origin.toLowerCase().includes(q))
      )
    })
  }

  // Most recent first
  return result.slice(-DISPLAY_LIMIT).reverse()
})

// --- Event type badge colours ---
const TYPE_COLORS: Record<string, string> = {
  request_created: '#6366f1',
  request_arrived: '#0ea5e9',
  request_started: '#0ea5e9',
  request_completed: '#059669',
  request_timed_out: '#b91c1c',
  retry_scheduled: '#f59e0b',
  node_failed: '#dc2626',
  node_recovered: '#059669',
  message_queued: '#a1a1aa',
  message_dropped: '#dc2626',
  request_shedded: '#ef4444',
  request_dequeued: '#0ea5e9',
  connection_failed: '#dc2626',
  connection_restored: '#059669',
  cache_hit: '#059669',
  cache_miss: '#f59e0b',
  stale_read: '#ef4444',
}

function typeColor(type: string): string {
  return TYPE_COLORS[type] || '#6b7280'
}

function formatTime(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(2)}s`
}

function eventLabel(type: string): string {
  return type.replace(/_/g, ' ')
}

function eventDetail(
  e: {
    type: string
    request_id?: number
    node_id?: string
    origin?: string
    from?: string
    to?: string
    success?: boolean
    queue_id?: string
    retry_count?: number
    policy?: string
  },
): string {
  const parts: string[] = []
  if (e.request_id !== undefined) parts.push(`req#${e.request_id}`)
  if (e.node_id) parts.push(e.node_id)
  if (e.origin) parts.push(`from ${e.origin}`)
  if (e.from && e.to) parts.push(`${e.from}->${e.to}`)
  if (e.success !== undefined) parts.push(e.success ? 'success' : 'failed')
  if (e.retry_count !== undefined) parts.push(`retry#${e.retry_count}`)
  if (e.queue_id) parts.push(`queue:${e.queue_id}`)
  if (e.policy) parts.push(e.policy)
  return parts.join(' · ')
}

function clearFilters(): void {
  searchText.value = ''
  filterType.value = 'all'
  filterNodeId.value = 'all'
  filterRequestId.value = ''
}
</script>

<template>
  <div class="timeline">
    <div class="timeline__header">
      <h3 class="timeline__title">Event Timeline</h3>
      <span class="timeline__count">{{ sim.eventLog.length }} events</span>
    </div>

    <!-- Filters -->
    <div class="timeline__filters">
      <input
        v-model="searchText"
        class="timeline__search"
        type="text"
        placeholder="Search events…"
      />
      <select v-model="filterType" class="timeline__select">
        <option value="all">All types</option>
        <option v-for="t in eventTypes" :key="t" :value="t">
          {{ eventLabel(t) }}
        </option>
      </select>
      <select v-model="filterNodeId" class="timeline__select">
        <option value="all">All nodes</option>
        <option v-for="id in nodeIds" :key="id" :value="id">{{ id }}</option>
      </select>
      <input
        v-model="filterRequestId"
        class="timeline__req-filter"
        type="text"
        placeholder="Req ID"
      />
      <button
        v-if="searchText || filterType !== 'all' || filterNodeId !== 'all' || filterRequestId"
        class="timeline__clear"
        @click="clearFilters"
      >
        Clear
      </button>
    </div>

    <!-- Event list -->
    <div class="timeline__list">
      <div v-if="filteredEvents.length === 0" class="timeline__empty">
        <span v-if="sim.eventLog.length === 0">No events yet. Start a simulation to see events.</span>
        <span v-else>No events match the current filters.</span>
      </div>
      <div
        v-for="(evt, i) in filteredEvents"
        :key="i"
        class="timeline__entry"
      >
        <span
          class="timeline__badge"
          :style="{ background: typeColor(evt.event.type) }"
        ></span>
        <span class="timeline__time">{{ formatTime(evt.time) }}</span>
        <span class="timeline__type">{{ eventLabel(evt.event.type) }}</span>
        <span class="timeline__detail">{{ eventDetail(evt.event) }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.timeline {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background: var(--fl-bg);
}

.timeline__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--fl-space-2) var(--fl-space-3);
  border-bottom: 1px solid var(--fl-border);
  flex-shrink: 0;
}

.timeline__title {
  font-size: var(--fl-size-14);
  font-weight: 700;
  color: var(--fl-text);
  margin: 0;
}

.timeline__count {
  font-size: var(--fl-size-14);
  color: var(--fl-grey-3);
  font-variant-numeric: tabular-nums;
}

.timeline__filters {
  display: flex;
  gap: var(--fl-space-1);
  padding: var(--fl-space-1) var(--fl-space-2);
  border-bottom: 1px solid var(--fl-border);
  flex-shrink: 0;
}

.timeline__search {
  flex: 1;
  min-width: 0;
  padding: 4px 8px;
  background: var(--fl-bg-alt);
  border: 1px solid var(--fl-border);
  border-radius: 4px;
  color: var(--fl-text);
  font-size: var(--fl-size-14);
}

.timeline__search:focus {
  outline: none;
  border-color: var(--fl-amber);
}

.timeline__select {
  padding: 4px 8px;
  background: var(--fl-bg-alt);
  border: 1px solid var(--fl-border);
  border-radius: 4px;
  color: var(--fl-text);
  font-size: var(--fl-size-14);
  cursor: pointer;
}

.timeline__req-filter {
  width: 64px;
  padding: 4px 8px;
  background: var(--fl-bg-alt);
  border: 1px solid var(--fl-border);
  border-radius: 4px;
  color: var(--fl-text);
  font-size: var(--fl-size-14);
}

.timeline__req-filter:focus {
  outline: none;
  border-color: var(--fl-amber);
}

.timeline__clear {
  padding: 4px 8px;
  background: transparent;
  border: 1px solid var(--fl-border);
  border-radius: 4px;
  color: var(--fl-grey-3);
  font-size: var(--fl-size-14);
  cursor: pointer;
}

.timeline__clear:hover {
  color: var(--fl-amber);
  border-color: var(--fl-amber);
}

.timeline__list {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.timeline__empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--fl-grey-3);
  font-size: var(--fl-size-14);
  text-align: center;
  padding: var(--fl-space-4);
}

.timeline__entry {
  display: flex;
  align-items: center;
  gap: var(--fl-space-2);
  padding: 3px var(--fl-space-2);
  border-bottom: 1px solid var(--fl-border);
  font-size: var(--fl-size-14);
  line-height: 1.4;
}

.timeline__entry:hover {
  background: var(--fl-bg-alt);
}

.timeline__badge {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.timeline__time {
  color: var(--fl-grey-3);
  font-family: var(--fl-font-mono);
  font-size: 0.75rem;
  min-width: 56px;
  flex-shrink: 0;
}

.timeline__type {
  color: var(--fl-text);
  font-weight: 600;
  min-width: 100px;
  flex-shrink: 0;
}

.timeline__detail {
  color: var(--fl-text-secondary);
  font-size: 0.75rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
