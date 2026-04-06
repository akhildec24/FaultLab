<script setup lang="ts">
/**
 * SimulationControls — Run/Pause/Step/Reset buttons plus
 * live status (running, time, pending events, metrics).
 *
 * Converts the visual graph to a scenario JSON, validates it,
 * and sends it to the simulation worker via the simulation store.
 */

import { computed, ref } from 'vue'
import { useGraphStore } from '@/stores/graph'
import { useSimulationStore } from '@/stores/simulation'
import { useAnimationStore } from '@/stores/animation'
import { validateGraph, graphToScenarioJson } from '@/graph/converter'
import type { SpeedMultiplier } from '@/graph/animation'
import { Play, Pause, StepForward, FastForward, RotateCcw, AlertTriangle } from '@lucide/vue'
import FailurePanel from '@/components/FailurePanel.vue'

const graph = useGraphStore()
const sim = useSimulationStore()
const animation = useAnimationStore()

const showErrors = ref(false)
const isRunningLoop = ref(false)

const validation = computed(() =>
  validateGraph(graph.nodes, graph.edges),
)

const canRun = computed(() => validation.value.valid && graph.nodeCount > 0)

async function startSimulation() {
  if (!validation.value.valid) {
    showErrors.value = true
    return
  }
  showErrors.value = false
  const json = graphToScenarioJson(graph.nodes, graph.edges)
  await sim.loadScenario(json)
  if (sim.error || !sim.workerHealthy) return
  await sim.start()
  if (sim.error) return
  // Start continuous run loop
  isRunningLoop.value = true
  runLoop()
}

async function runLoop() {
  while (isRunningLoop.value && sim.running && !sim.error) {
    const steps = await sim.run(500)
    if (steps === 0) {
 // No more events to process — engine exhausted
 break
    }
 // Small yield to let UI update
 await new Promise((r) => setTimeout(r, 16))
  }
  isRunningLoop.value = false
}

async function pauseSimulation() {
  isRunningLoop.value = false
  await sim.pause()
}

async function stepSimulation() {
  await sim.step()
}

async function resetSimulation() {
  isRunningLoop.value = false
  await sim.reset()
}

async function continueRun() {
  if (!sim.loaded) return
  isRunningLoop.value = true
  runLoop()
}

const metrics = computed(() => sim.metrics)

const speedOptions: SpeedMultiplier[] = [0.5, 1, 2, 4]

function setSpeed(s: SpeedMultiplier) {
  animation.setSpeed(s)
}

const simClock = computed(() => {
  const ms = sim.currentTime
  if (ms < 1000) return `${ms}ms`
  const s = (ms / 1000).toFixed(1)
  return `${s}s`
})
</script>

<template>
  <div class="sim-controls">
    <!-- Validation errors -->
    <div class="sim-controls__errors" v-if="showErrors && !validation.valid">
      <div class="sim-controls__error" v-for="err in validation.errors" :key="err">
        <AlertTriangle :size="16" /> {{ err }}
      </div>
    </div>

    <!-- Validation warnings -->
    <div class="sim-controls__warnings" v-if="validation.warnings.length > 0 && !showErrors">
      <div class="sim-controls__warning" v-for="w in validation.warnings" :key="w">
        {{ w }}
      </div>
    </div>

    <!-- Sim errors -->
    <div class="sim-controls__errors" v-if="sim.error">
      <div class="sim-controls__error"><AlertTriangle :size="16" /> {{ sim.error }}</div>
    </div>

    <!-- Buttons -->
    <div class="sim-controls__buttons">
      <button
        class="fl-button fl-button--primary sim-controls__btn"
        :disabled="sim.running"
        @click="startSimulation"
      >
        <Play :size="16" /> Run
      </button>
      <button
        class="fl-button fl-button--secondary sim-controls__btn"
        :disabled="!sim.running"
        @click="pauseSimulation"
      >
        <Pause :size="16" /> Pause
      </button>
      <button
        class="fl-button fl-button--secondary sim-controls__btn"
        :disabled="sim.running"
        @click="stepSimulation"
      >
        <StepForward :size="16" /> Step
      </button>
      <button
        class="fl-button fl-button--secondary sim-controls__btn"
        :disabled="sim.running"
        @click="continueRun"
      >
        <FastForward :size="16" /> Run 500
      </button>
      <button
        class="fl-button fl-button--warning sim-controls__btn"
        @click="resetSimulation"
      >
        <RotateCcw :size="16" /> Reset
      </button>

      <div class="sim-controls__speed">
        <span class="sim-controls__speed-label">Speed</span>
        <button
          v-for="s in speedOptions"
          :key="s"
          :class="['sim-controls__speed-btn', { 'is-active': animation.speed === s }]"
          @click="setSpeed(s)"
        >{{ s }}x</button>
      </div>
    </div>

    <!-- Status bar -->
    <div class="sim-controls__status" v-if="sim.loaded">
      <div class="sim-controls__status-item">
        <span class="sim-controls__status-label">State</span>
        <span :class="['sim-controls__status-value', sim.running ? 'is-running' : 'is-paused']">
          {{ sim.running ? 'Running' : 'Paused' }}
        </span>
      </div>
      <div class="sim-controls__status-item">
        <span class="sim-controls__status-label">Clock</span>
        <span class="sim-controls__status-value sim-controls__clock">{{ simClock }}</span>
      </div>
      <div class="sim-controls__status-item">
        <span class="sim-controls__status-label">Pending</span>
        <span class="sim-controls__status-value">{{ sim.pendingEvents }}</span>
      </div>
    </div>

    <!-- Metrics + Failure injection side by side -->
    <div class="sim-controls__row" v-if="sim.loaded">
      <div class="sim-controls__metrics" v-if="metrics">
        <div class="sim-controls__metric">
          <span class="sim-controls__metric-label">Requests</span>
          <span class="sim-controls__metric-value">{{ metrics.total_requests }}</span>
        </div>
        <div class="sim-controls__metric">
          <span class="sim-controls__metric-label">Success</span>
          <span class="sim-controls__metric-value sim-controls__metric--ok">{{ metrics.successful }}</span>
        </div>
        <div class="sim-controls__metric">
          <span class="sim-controls__metric-label">Failed</span>
          <span class="sim-controls__metric-value sim-controls__metric--err">{{ metrics.failed }}</span>
        </div>
        <div class="sim-controls__metric">
          <span class="sim-controls__metric-label">Timed out</span>
          <span class="sim-controls__metric-value sim-controls__metric--err">{{ metrics.timed_out }}</span>
        </div>
        <div class="sim-controls__metric">
          <span class="sim-controls__metric-label">Dropped</span>
          <span class="sim-controls__metric-value sim-controls__metric--err">{{ metrics.dropped }}</span>
        </div>
        <div class="sim-controls__metric">
          <span class="sim-controls__metric-label">Avg latency</span>
          <span class="sim-controls__metric-value">{{ Math.round(metrics.avg_latency_ms) }}ms</span>
        </div>
        <div class="sim-controls__metric">
          <span class="sim-controls__metric-label">P95</span>
          <span class="sim-controls__metric-value">{{ Math.round(metrics.p95_latency_ms) }}ms</span>
        </div>
      </div>
      <FailurePanel />
    </div>
  </div>
</template>

<style scoped>
.sim-controls {
  display: flex;
  flex-direction: column;
  gap: var(--fl-space-2);
  padding: var(--fl-space-2) var(--fl-space-3);
  background: var(--fl-slate);
  flex-shrink: 0;
}

.sim-controls__buttons {
  display: flex;
  gap: var(--fl-space-1);
  flex-wrap: wrap;
}

.sim-controls__btn {
  font-size: var(--fl-size-14);
  padding: var(--fl-space-1) var(--fl-space-2);
  color: var(--fl-white);
  border-color: var(--fl-slate-light);
  background: transparent;
}

.sim-controls__btn:hover:not(:disabled) {
  background: var(--fl-slate-light);
  color: var(--fl-white);
}

.sim-controls__btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.sim-controls__speed {
  display: flex;
  align-items: center;
  gap: var(--fl-space-1);
  margin-left: auto;
}

.sim-controls__speed-label {
  font-size: var(--fl-size-14);
  color: var(--fl-grey-3);
}

.sim-controls__speed-btn {
  padding: var(--fl-space-1) var(--fl-space-2);
  font-size: var(--fl-size-14);
  font-weight: 600;
  color: var(--fl-grey-2);
  background: transparent;
  border: 1px solid var(--fl-slate-light);
  cursor: pointer;
  font-variant-numeric: tabular-nums;
}

.sim-controls__speed-btn:hover {
  color: var(--fl-white);
  background: var(--fl-slate-light);
}

.sim-controls__speed-btn.is-active {
  color: var(--fl-slate);
  background: var(--fl-amber);
  border-color: var(--fl-amber);
}

.sim-controls__errors {
  background: var(--fl-red);
  padding: var(--fl-space-1) var(--fl-space-2);
}

.sim-controls__error {
  color: var(--fl-white);
  font-size: var(--fl-size-14);
  font-weight: 600;
}

.sim-controls__warnings {
  padding: var(--fl-space-1) var(--fl-space-2);
  background: var(--fl-slate-light);
}

.sim-controls__warning {
  color: var(--fl-amber);
  font-size: var(--fl-size-14);
}

.sim-controls__status {
  display: flex;
  gap: var(--fl-space-4);
  padding: var(--fl-space-1) 0;
}

.sim-controls__status-item {
  display: flex;
  flex-direction: column;
}

.sim-controls__status-label {
  font-size: var(--fl-size-14);
  color: var(--fl-grey-3);
}

.sim-controls__status-value {
  font-size: var(--fl-size-16);
  color: var(--fl-white);
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.is-running {
  color: var(--fl-green);
}

.is-paused {
  color: var(--fl-amber);
}

.sim-controls__clock {
  font-variant-numeric: tabular-nums;
  color: var(--fl-amber);
}

.sim-controls__row {
  display: flex;
  gap: var(--fl-space-2);
  align-items: stretch;
}

.sim-controls__metrics {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
  gap: var(--fl-space-1);
  flex: 1;
}

.sim-controls__metric {
  display: flex;
  flex-direction: column;
  padding: var(--fl-space-1) var(--fl-space-2);
  background: var(--fl-slate-light);
}

.sim-controls__metric-label {
  font-size: var(--fl-size-14);
  color: var(--fl-grey-3);
}

.sim-controls__metric-value {
  font-size: var(--fl-size-16);
  color: var(--fl-white);
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.sim-controls__metric--ok {
  color: var(--fl-green);
}

.sim-controls__metric--err {
  color: var(--fl-red);
}
</style>
