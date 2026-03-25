/**
 * Stub WASM module — replaced by wasm-pack output.
 *
 * This file exists so that Vite's import analysis doesn't fail
 * when the real WASM glue code hasn't been generated yet.
 * Run `just wasm-pack` to generate the real module.
 */

export class Simulation {
  loadScenario(json) {
    console.warn('[stub] Simulation.loadScenario — run `just wasm-pack` to generate real WASM')
  }
  start() {
    console.warn('[stub] Simulation.start')
  }
  pause() {
    console.warn('[stub] Simulation.pause')
  }
  reset() {
    console.warn('[stub] Simulation.reset')
  }
  step() {
    console.warn('[stub] Simulation.step')
    return false
  }
  run(maxSteps) {
    console.warn('[stub] Simulation.run')
    return 0
  }
  isRunning() {
    return false
  }
  currentTime() {
    return 0
  }
  getMetrics() {
    return JSON.stringify({
      total_requests: 0,
      successful: 0,
      failed: 0,
      timed_out: 0,
      retries: 0,
      dropped: 0,
      current_rps: 0,
      avg_latency_ms: 0,
      p50_latency_ms: 0,
      p95_latency_ms: 0,
      p99_latency_ms: 0,
      queue_depths: {},
      node_utilisation: {},
    })
  }
  getState() {
    return JSON.stringify({ current_time: 0, nodes: {}, requests: {} })
  }
  getRecentEvents() {
    return '[]'
  }
  pendingEvents() {
    return 0
  }
  injectFailure(json) {
    console.warn('[stub] Simulation.injectFailure:', json)
  }
}
