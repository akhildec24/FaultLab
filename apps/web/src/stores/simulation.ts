/**
 * Pinia store for the simulation engine.
 *
 * Provides reactive state that mirrors the WASM engine running
 * inside a Web Worker. Components can call `loadScenario`, `start`,
 * `pause`, `step`, `run`, and `reset` — all of which delegate to
 * the worker and update reactive state on response.
 */

import { defineStore } from 'pinia'
import { ref, computed, shallowRef } from 'vue'
import { SimulationWorkerClient } from '@faultlab/simulation-client'
import type { SimulationEvent, Metrics } from '@faultlab/simulation-client'

export const useSimulationStore = defineStore('simulation', () => {
  // --- State ---
  const running = ref(false)
  const currentTime = ref(0)
  const pendingEvents = ref(0)
  const metrics = shallowRef<Metrics | null>(null)
  const recentEvents = shallowRef<unknown[]>([])
  const eventLog = ref<unknown[]>([])
  const EVENT_LOG_CAP = 5000
  const error = ref<string | null>(null)
  const loaded = ref(false)
  const workerHealthy = ref(true)
  let lastLoadedScenario: string | null = null
  let recoveryTimer: ReturnType<typeof setTimeout> | null = null
  const RECOVERY_BASE_DELAY = 1000
  const RECOVERY_MAX_DELAY = 10000

  // --- Worker client (lazy init) ---
  let client: SimulationWorkerClient | null = null

  /** Callback fired when new events arrive from the worker. */
  let eventsCallback: ((events: unknown[]) => void) | null = null

  function onEvents(cb: ((events: unknown[]) => void) | null): void {
    eventsCallback = cb
  }

  function ensureClient(): SimulationWorkerClient {
    if (!client) {
      const workerUrl = new URL(
        '@faultlab/simulation-client/worker',
        import.meta.url,
      )
      client = new SimulationWorkerClient(workerUrl)
      client.onEvent(handleEvent)
      // Monitor worker health
      client.onWorkerError(() => {
        workerHealthy.value = false
        scheduleWorkerRecovery()
      })
    }
    return client
  }

  function scheduleWorkerRecovery(): void {
    if (recoveryTimer) clearTimeout(recoveryTimer)
    const attempts = 0
    const delay = Math.min(RECOVERY_BASE_DELAY * Math.pow(2, attempts), RECOVERY_MAX_DELAY)
    recoveryTimer = setTimeout(() => recoverWorker(), delay)
  }

  async function recoverWorker(): Promise<void> {
    // Terminate old client
    if (client) {
      try { client.terminate() } catch { /* ignore */ }
      client = null
    }
    // Recreate and reload scenario if we had one
    if (lastLoadedScenario) {
      try {
        const c = ensureClient()
        await c.loadScenario(lastLoadedScenario)
        loaded.value = true
        workerHealthy.value = true
        await refreshStatus()
      } catch {
        // Retry with backoff
        scheduleWorkerRecovery()
      }
    } else {
      workerHealthy.value = true
    }
  }

  function handleEvent(event: SimulationEvent): void {
    switch (event.type) {
      case 'EVENTS':
        recentEvents.value = event.events
        // Accumulate into event log with cap
        const combined = eventLog.value.concat(event.events)
        eventLog.value = combined.length > EVENT_LOG_CAP
          ? combined.slice(combined.length - EVENT_LOG_CAP)
          : combined
        if (eventsCallback) eventsCallback(event.events)
        break
      case 'ENGINE_STOPPED':
        running.value = false
        break
    }
  }

  function clearError() {
    error.value = null
  }

  // --- Actions ---

  async function loadScenario(json: string): Promise<void> {
    clearError()
    try {
      const c = ensureClient()
      lastLoadedScenario = json
      await c.loadScenario(json)
      loaded.value = true
      workerHealthy.value = true
      await refreshStatus()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function start(): Promise<void> {
    clearError()
    try {
      const c = ensureClient()
      await c.start()
      running.value = true
      await refreshStatus()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function pause(): Promise<void> {
    clearError()
    try {
      const c = ensureClient()
      await c.pause()
      running.value = false
      await refreshStatus()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function reset(): Promise<void> {
    clearError()
    try {
      const c = ensureClient()
      await c.reset()
      running.value = false
      currentTime.value = 0
      recentEvents.value = []
      eventLog.value = []
      await refreshStatus()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function step(): Promise<boolean> {
    clearError()
    try {
      const c = ensureClient()
      const processed = await c.step()
      await refreshStatus()
      return processed
    } catch (e) {
      error.value = String(e)
      return false
    }
  }

  async function run(maxSteps: number): Promise<number> {
    clearError()
    try {
      const c = ensureClient()
      const steps = await c.run(maxSteps)
      await refreshStatus()
      return steps
    } catch (e) {
      error.value = String(e)
      return 0
    }
  }

  async function injectFailure(json: string): Promise<void> {
    clearError()
    try {
      const c = ensureClient()
      await c.injectFailure(json)
      await refreshStatus()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function refreshMetrics(): Promise<void> {
    if (!client) return
    try {
      const json = await client.getMetrics()
      metrics.value = JSON.parse(json) as Metrics
    } catch {
      // Ignore
    }
  }

  async function refreshStatus(): Promise<void> {
    if (!client) return
    try {
      const status = await client.getStatus()
      running.value = status.running
      currentTime.value = status.currentTime
      pendingEvents.value = status.pendingEvents
      await refreshMetrics()
    } catch {
      // Ignore
    }
  }

  function terminate(): void {
    if (client) {
      client.terminate()
      client = null
    }
    running.value = false
    currentTime.value = 0
    pendingEvents.value = 0
    metrics.value = null
    recentEvents.value = []
    eventLog.value = []
    loaded.value = false
  }

  // --- Computed ---
  const isLoaded = computed(() => loaded.value)
  const hasError = computed(() => error.value !== null)

  return {
    // State
    running,
    currentTime,
    pendingEvents,
    metrics,
    recentEvents,
    eventLog,
    error,
    loaded,
    workerHealthy,
    // Computed
    isLoaded,
    hasError,
    // Actions
    loadScenario,
    start,
    pause,
    reset,
    step,
    run,
    injectFailure,
    refreshMetrics,
    refreshStatus,
    terminate,
    onEvents,
  }
})
