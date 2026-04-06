<script setup lang="ts">
/**
 * Dashboard — real-time metrics dashboard with throughput, error rate,
 * latency percentiles, queue depth, and node utilisation.
 *
 * Also supports snapshot comparison: take a snapshot of current metrics,
 * run another simulation, and compare side-by-side.
 */

import { computed, ref } from 'vue'
import { useSimulationStore } from '@/stores/simulation'
import type { Metrics } from '@faultlab/simulation-client'
import { Camera, ArrowRight } from '@lucide/vue'

const sim = useSimulationStore()

// --- Snapshot for comparison ---
interface Snapshot {
  label: string
  metrics: Metrics
  time: number
}

const snapshots = ref<Snapshot[]>([])

function takeSnapshot(): void {
  if (!sim.metrics) return
  snapshots.value.push({
    label: `Snapshot ${snapshots.value.length + 1}`,
    metrics: { ...sim.metrics },
    time: sim.currentTime,
  })
}

function clearSnapshots(): void {
  snapshots.value = []
}

function removeSnapshot(idx: number): void {
  snapshots.value.splice(idx, 1)
}

// --- Derived metrics ---
const m = computed(() => sim.metrics)

const errorRate = computed(() => {
  const met = m.value
  if (!met || met.total_requests === 0) return 0
  return ((met.failed + met.timed_out) / met.total_requests) * 100
})

const successRate = computed(() => {
  const met = m.value
  if (!met || met.total_requests === 0) return 0
  return (met.successful / met.total_requests) * 100
})

const cacheHitRate = computed(() => {
  const met = m.value
  if (!met) return 0
  const total = met.cache_hits + met.cache_misses
  if (total === 0) return null
  return (met.cache_hits / total) * 100
})

const throughput = computed(() => {
  const met = m.value
  if (!met) return 0
  return met.current_rps
})

// --- Queue depths as sorted array ---
const queueDepths = computed(() => {
  const met = m.value
  if (!met || !met.queue_depths) return []
  return Object.entries(met.queue_depths)
    .map(([node, depth]) => ({ node, depth }))
    .sort((a, b) => b.depth - a.depth)
})

// --- Node utilisation as sorted array ---
const nodeUtilisation = computed(() => {
  const met = m.value
  if (!met || !met.node_utilisation) return []
  return Object.entries(met.node_utilisation)
    .map(([node, util]) => ({ node, util: util * 100 }))
    .sort((a, b) => b.util - a.util)
})

// --- Max queue depth for bar scaling ---
const maxQueueDepth = computed(() => {
  const max = Math.max(...queueDepths.value.map((q) => q.depth), 1)
  return Math.max(max, 10)
})

// --- Comparison: diff between two snapshots ---
const comparison = computed(() => {
  if (snapshots.value.length < 2) return null
  const a = snapshots.value[snapshots.value.length - 2]
  const b = snapshots.value[snapshots.value.length - 1]
  const diff = (av: number, bv: number) => bv - av
  const pctDiff = (av: number, bv: number) => {
    if (av === 0) return bv > 0 ? 100 : 0
    return ((bv - av) / av) * 100
  }
  return {
    a,
    b,
    deltas: {
      total_requests: diff(a.metrics.total_requests, b.metrics.total_requests),
      successful: diff(a.metrics.successful, b.metrics.successful),
      failed: diff(a.metrics.failed, b.metrics.failed),
      timed_out: diff(a.metrics.timed_out, b.metrics.timed_out),
      avg_latency_ms: Math.round(diff(a.metrics.avg_latency_ms, b.metrics.avg_latency_ms) * 10) / 10,
      p95_latency_ms: Math.round(diff(a.metrics.p95_latency_ms, b.metrics.p95_latency_ms) * 10) / 10,
      current_rps: Math.round(diff(a.metrics.current_rps, b.metrics.current_rps) * 10) / 10,
      error_rate: Math.round(pctDiff(a.metrics.failed + a.metrics.timed_out, b.metrics.failed + b.metrics.timed_out) * 10) / 10,
    },
  }
})

function fmtMs(ms: number): string {
  if (ms < 10) return ms.toFixed(1)
  return Math.round(ms).toString()
}

function fmtPct(pct: number): string {
  return pct.toFixed(1) + '%'
}

function fmtDelta(delta: number): string {
  const sign = delta > 0 ? '+' : ''
  return sign + delta.toString()
}

function deltaClass(delta: number): string {
  if (delta > 0) return 'dashboard__delta--up'
  if (delta < 0) return 'dashboard__delta--down'
  return 'dashboard__delta--flat'
}
</script>

<template>
  <div class="dashboard">
    <div class="dashboard__header">
      <h3 class="dashboard__title">Dashboard</h3>
      <div class="dashboard__actions">
        <button
          class="dashboard__btn"
          :disabled="!sim.metrics"
          @click="takeSnapshot"
        >
          <Camera :size="16" /> Snapshot
        </button>
        <button
          v-if="snapshots.length > 0"
          class="dashboard__btn"
          @click="clearSnapshots"
        >
          Clear
        </button>
      </div>
    </div>

    <div v-if="!m" class="dashboard__empty">
      No metrics yet. Start a simulation to see the dashboard.
    </div>

    <div v-else class="dashboard__content">
      <!-- KPI cards row -->
      <div class="dashboard__kpis">
        <div class="dashboard__kpi">
          <span class="dashboard__kpi-label">Throughput</span>
          <span class="dashboard__kpi-value">{{ throughput.toFixed(1) }} rps</span>
        </div>
        <div class="dashboard__kpi">
          <span class="dashboard__kpi-label">Success rate</span>
          <span class="dashboard__kpi-value dashboard__kpi--ok">{{ fmtPct(successRate) }}</span>
        </div>
        <div class="dashboard__kpi">
          <span class="dashboard__kpi-label">Error rate</span>
          <span class="dashboard__kpi-value" :class="errorRate > 5 ? 'dashboard__kpi--err' : ''">
            {{ fmtPct(errorRate) }}
          </span>
        </div>
        <div class="dashboard__kpi">
          <span class="dashboard__kpi-label">Avg latency</span>
          <span class="dashboard__kpi-value">{{ fmtMs(m.avg_latency_ms) }}ms</span>
        </div>
        <div class="dashboard__kpi">
          <span class="dashboard__kpi-label">P50</span>
          <span class="dashboard__kpi-value">{{ fmtMs(m.p50_latency_ms) }}ms</span>
        </div>
        <div class="dashboard__kpi">
          <span class="dashboard__kpi-label">P95</span>
          <span class="dashboard__kpi-value">{{ fmtMs(m.p95_latency_ms) }}ms</span>
        </div>
        <div class="dashboard__kpi">
          <span class="dashboard__kpi-label">P99</span>
          <span class="dashboard__kpi-value">{{ fmtMs(m.p99_latency_ms) }}ms</span>
        </div>
        <div class="dashboard__kpi" v-if="cacheHitRate !== null">
          <span class="dashboard__kpi-label">Cache hit</span>
          <span class="dashboard__kpi-value">{{ fmtPct(cacheHitRate) }}</span>
        </div>
      </div>

      <!-- Request counts row -->
      <div class="dashboard__section">
        <h4 class="dashboard__section-title">Request Counts</h4>
        <div class="dashboard__counts">
          <div class="dashboard__count">
            <span class="dashboard__count-label">Total</span>
            <span class="dashboard__count-value">{{ m.total_requests }}</span>
          </div>
          <div class="dashboard__count">
            <span class="dashboard__count-label">Success</span>
            <span class="dashboard__count-value dashboard__count--ok">{{ m.successful }}</span>
          </div>
          <div class="dashboard__count">
            <span class="dashboard__count-label">Failed</span>
            <span class="dashboard__count-value dashboard__count--err">{{ m.failed }}</span>
          </div>
          <div class="dashboard__count">
            <span class="dashboard__count-label">Timed out</span>
            <span class="dashboard__count-value dashboard__count--err">{{ m.timed_out }}</span>
          </div>
          <div class="dashboard__count">
            <span class="dashboard__count-label">Dropped</span>
            <span class="dashboard__count-value dashboard__count--err">{{ m.dropped }}</span>
          </div>
          <div class="dashboard__count">
            <span class="dashboard__count-label">Shedded</span>
            <span class="dashboard__count-value dashboard__count--warn">{{ m.shedded }}</span>
          </div>
          <div class="dashboard__count">
            <span class="dashboard__count-label">Retries</span>
            <span class="dashboard__count-value dashboard__count--warn">{{ m.retries }}</span>
          </div>
          <div class="dashboard__count">
            <span class="dashboard__count-label">Stale reads</span>
            <span class="dashboard__count-value dashboard__count--err">{{ m.stale_reads }}</span>
          </div>
        </div>
      </div>

      <!-- Queue depths -->
      <div class="dashboard__section" v-if="queueDepths.length > 0">
        <h4 class="dashboard__section-title">Queue Depths</h4>
        <div class="dashboard__bars">
          <div
            v-for="q in queueDepths"
            :key="q.node"
            class="dashboard__bar-row"
          >
            <span class="dashboard__bar-label">{{ q.node }}</span>
            <div class="dashboard__bar-track">
              <div
                class="dashboard__bar-fill dashboard__bar-fill--queue"
                :style="{ width: (q.depth / maxQueueDepth * 100) + '%' }"
              ></div>
            </div>
            <span class="dashboard__bar-value">{{ q.depth }}</span>
          </div>
        </div>
      </div>

      <!-- Node utilisation -->
      <div class="dashboard__section" v-if="nodeUtilisation.length > 0">
        <h4 class="dashboard__section-title">Node Utilisation</h4>
        <div class="dashboard__bars">
          <div
            v-for="n in nodeUtilisation"
            :key="n.node"
            class="dashboard__bar-row"
          >
            <span class="dashboard__bar-label">{{ n.node }}</span>
            <div class="dashboard__bar-track">
              <div
                class="dashboard__bar-fill"
                :class="n.util > 80 ? 'dashboard__bar-fill--hot' : n.util > 50 ? 'dashboard__bar-fill--warm' : 'dashboard__bar-fill--cool'"
                :style="{ width: n.util + '%' }"
              ></div>
            </div>
            <span class="dashboard__bar-value">{{ n.util.toFixed(0) }}%</span>
          </div>
        </div>
      </div>

      <!-- Snapshots list -->
      <div class="dashboard__section" v-if="snapshots.length > 0">
        <h4 class="dashboard__section-title">Snapshots ({{ snapshots.length }})</h4>
        <div class="dashboard__snapshots">
          <div
            v-for="(snap, i) in snapshots"
            :key="i"
            class="dashboard__snapshot"
          >
            <div class="dashboard__snapshot-header">
              <span class="dashboard__snapshot-label">{{ snap.label }}</span>
              <button class="dashboard__snapshot-remove" @click="removeSnapshot(i)">×</button>
            </div>
            <div class="dashboard__snapshot-metrics">
              <span>{{ snap.metrics.total_requests }} reqs</span>
              <span>{{ snap.metrics.successful }} ok</span>
              <span>{{ snap.metrics.failed }} fail</span>
              <span>{{ fmtMs(snap.metrics.p95_latency_ms) }}ms p95</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Comparison -->
      <div class="dashboard__section" v-if="comparison">
        <h4 class="dashboard__section-title">
          Comparison: {{ comparison.a.label }} <ArrowRight :size="14" /> {{ comparison.b.label }}
        </h4>
        <div class="dashboard__comparison">
          <div class="dashboard__comp-row">
            <span class="dashboard__comp-label">Total requests</span>
            <span :class="['dashboard__comp-delta', deltaClass(comparison.deltas.total_requests)]">
              {{ fmtDelta(comparison.deltas.total_requests) }}
            </span>
          </div>
          <div class="dashboard__comp-row">
            <span class="dashboard__comp-label">Successful</span>
            <span :class="['dashboard__comp-delta', deltaClass(comparison.deltas.successful)]">
              {{ fmtDelta(comparison.deltas.successful) }}
            </span>
          </div>
          <div class="dashboard__comp-row">
            <span class="dashboard__comp-label">Failed</span>
            <span :class="['dashboard__comp-delta', deltaClass(-comparison.deltas.failed)]">
              {{ fmtDelta(comparison.deltas.failed) }}
            </span>
          </div>
          <div class="dashboard__comp-row">
            <span class="dashboard__comp-label">Timed out</span>
            <span :class="['dashboard__comp-delta', deltaClass(-comparison.deltas.timed_out)]">
              {{ fmtDelta(comparison.deltas.timed_out) }}
            </span>
          </div>
          <div class="dashboard__comp-row">
            <span class="dashboard__comp-label">Avg latency</span>
            <span :class="['dashboard__comp-delta', deltaClass(-comparison.deltas.avg_latency_ms)]">
              {{ fmtDelta(comparison.deltas.avg_latency_ms) }}ms
            </span>
          </div>
          <div class="dashboard__comp-row">
            <span class="dashboard__comp-label">P95 latency</span>
            <span :class="['dashboard__comp-delta', deltaClass(-comparison.deltas.p95_latency_ms)]">
              {{ fmtDelta(comparison.deltas.p95_latency_ms) }}ms
            </span>
          </div>
          <div class="dashboard__comp-row">
            <span class="dashboard__comp-label">Throughput</span>
            <span :class="['dashboard__comp-delta', deltaClass(comparison.deltas.current_rps)]">
              {{ fmtDelta(comparison.deltas.current_rps) }} rps
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dashboard {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background: var(--fl-bg);
}

.dashboard__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--fl-space-2) var(--fl-space-3);
  border-bottom: 1px solid var(--fl-border);
  flex-shrink: 0;
}

.dashboard__title {
  font-size: var(--fl-size-14);
  font-weight: 700;
  color: var(--fl-text);
  margin: 0;
}

.dashboard__actions {
  display: flex;
  gap: var(--fl-space-1);
}

.dashboard__btn {
  padding: 4px 8px;
  background: transparent;
  border: 1px solid var(--fl-border);
  border-radius: 4px;
  color: var(--fl-text-secondary);
  font-size: var(--fl-size-14);
  cursor: pointer;
}

.dashboard__btn:hover:not(:disabled) {
  color: var(--fl-amber);
  border-color: var(--fl-amber);
}

.dashboard__btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.dashboard__empty {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: var(--fl-grey-3);
  font-size: var(--fl-size-14);
  text-align: center;
  padding: var(--fl-space-4);
}

.dashboard__content {
  flex: 1;
  overflow-y: auto;
  padding: var(--fl-space-2) var(--fl-space-3);
  display: flex;
  flex-direction: column;
  gap: var(--fl-space-3);
}

/* KPI cards */
.dashboard__kpis {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
  gap: var(--fl-space-1);
}

.dashboard__kpi {
  display: flex;
  flex-direction: column;
  padding: var(--fl-space-1) var(--fl-space-2);
  background: var(--fl-bg-alt);
  border: 1px solid var(--fl-border);
  border-radius: 4px;
}

.dashboard__kpi-label {
  font-size: 0.75rem;
  color: var(--fl-grey-3);
}

.dashboard__kpi-value {
  font-size: var(--fl-size-16);
  font-weight: 700;
  color: var(--fl-text);
  font-variant-numeric: tabular-nums;
}

.dashboard__kpi--ok {
  color: var(--fl-green);
}

.dashboard__kpi--err {
  color: var(--fl-red);
}

/* Sections */
.dashboard__section {
  display: flex;
  flex-direction: column;
  gap: var(--fl-space-1);
}

.dashboard__section-title {
  font-size: var(--fl-size-14);
  font-weight: 700;
  color: var(--fl-text);
  margin: 0;
  padding-bottom: var(--fl-space-1);
  border-bottom: 1px solid var(--fl-border);
}

/* Request counts */
.dashboard__counts {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
  gap: var(--fl-space-1);
}

.dashboard__count {
  display: flex;
  flex-direction: column;
  padding: var(--fl-space-1) var(--fl-space-2);
  background: var(--fl-bg-alt);
  border-radius: 4px;
}

.dashboard__count-label {
  font-size: 0.75rem;
  color: var(--fl-grey-3);
}

.dashboard__count-value {
  font-size: var(--fl-size-16);
  font-weight: 700;
  color: var(--fl-text);
  font-variant-numeric: tabular-nums;
}

.dashboard__count--ok { color: var(--fl-green); }
.dashboard__count--err { color: var(--fl-red); }
.dashboard__count--warn { color: var(--fl-amber); }

/* Bar charts */
.dashboard__bars {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.dashboard__bar-row {
  display: flex;
  align-items: center;
  gap: var(--fl-space-2);
}

.dashboard__bar-label {
  width: 80px;
  font-size: 0.75rem;
  color: var(--fl-text-secondary);
  flex-shrink: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dashboard__bar-track {
  flex: 1;
  height: 16px;
  background: var(--fl-bg-alt);
  border-radius: 2px;
  overflow: hidden;
}

.dashboard__bar-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.3s ease;
}

.dashboard__bar-fill--queue { background: var(--fl-amber); }
.dashboard__bar-fill--cool { background: var(--fl-green); }
.dashboard__bar-fill--warm { background: var(--fl-amber); }
.dashboard__bar-fill--hot { background: var(--fl-red); }

.dashboard__bar-value {
  width: 48px;
  font-size: 0.75rem;
  color: var(--fl-text);
  font-variant-numeric: tabular-nums;
  text-align: right;
  flex-shrink: 0;
}

/* Snapshots */
.dashboard__snapshots {
  display: flex;
  flex-direction: column;
  gap: var(--fl-space-1);
}

.dashboard__snapshot {
  padding: var(--fl-space-1) var(--fl-space-2);
  background: var(--fl-bg-alt);
  border: 1px solid var(--fl-border);
  border-radius: 4px;
}

.dashboard__snapshot-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.dashboard__snapshot-label {
  font-size: var(--fl-size-14);
  font-weight: 600;
  color: var(--fl-text);
}

.dashboard__snapshot-remove {
  background: none;
  border: none;
  color: var(--fl-grey-3);
  cursor: pointer;
  font-size: var(--fl-size-16);
  padding: 0 4px;
}

.dashboard__snapshot-remove:hover {
  color: var(--fl-red);
}

.dashboard__snapshot-metrics {
  display: flex;
  gap: var(--fl-space-3);
  margin-top: 2px;
  font-size: 0.75rem;
  color: var(--fl-text-secondary);
  font-variant-numeric: tabular-nums;
}

/* Comparison */
.dashboard__comparison {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.dashboard__comp-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 3px var(--fl-space-2);
  background: var(--fl-bg-alt);
  border-radius: 2px;
}

.dashboard__comp-label {
  font-size: var(--fl-size-14);
  color: var(--fl-text-secondary);
}

.dashboard__comp-delta {
  font-size: var(--fl-size-14);
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.dashboard__delta--up { color: var(--fl-green); }
.dashboard__delta--down { color: var(--fl-red); }
.dashboard__delta--flat { color: var(--fl-grey-3); }
</style>
