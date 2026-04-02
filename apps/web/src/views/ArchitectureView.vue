<script setup lang="ts">
import { RouterLink } from 'vue-router'

const layers = [
  {
    name: 'Frontend',
    tag: 'Interface',
    tech: 'Vue 3, TypeScript, Vite, Pinia',
    desc: 'Reactive UI with graph editor (SVG), code editor, dashboards, event timeline, and multiplayer presence.',
    points: [
      'SVG-based graph editor with pan/zoom, drag, connect, viewport culling',
      'Pinia stores mirror worker state reactively',
      'Scenario language with syntax highlighting and error underlining',
      'WebSocket client composable for real-time collaboration',
    ],
  },
  {
    name: 'Web Worker',
    tag: 'Bridge',
    tech: 'TypeScript Worker, promise-based client',
    desc: 'Off-main-thread execution keeps the UI responsive during simulation.',
    points: [
      'Typed message protocol with request IDs for response matching',
      'Unsolicited EVENTS messages pushed after each step/run',
      'Worker crash detection with automatic recovery and scenario reload',
      'Exponential backoff reconnection (1s → 10s max)',
    ],
  },
  {
    name: 'Simulation Engine',
    tag: 'Core',
    tech: 'Rust → WebAssembly',
    desc: 'Deterministic discrete-event simulation engine compiled to WASM.',
    points: [
      'Request lifecycle: generate → queue → process → respond → retry',
      'Configurable: capacity, latency, error rate, timeouts, retry policies',
      'Failure injection: crashes, latency spikes, network partitions',
      'Replication: leader/replica with configurable lag',
      '146 unit tests, clippy clean',
    ],
  },
  {
    name: 'Local-First Storage',
    tag: 'Persistence',
    tech: 'IndexedDB, Automerge CRDT',
    desc: 'Scenarios persist locally, work offline, and sync across devices.',
    points: [
      'Auto-save with 2s debounce',
      'History snapshots every 30s + manual snapshots',
      'Import/export JSON for sharing scenarios',
      'Online/offline indicator',
    ],
  },
  {
    name: 'Collaboration Server',
    tag: 'Multiplayer',
    tech: 'Gleam, Erlang/OTP, mist WebSocket',
    desc: 'Actor-based WebSocket server for real-time multiplayer editing.',
    points: [
      'Room management with OTP actors and named subjects',
      'Presence: join/leave, names, colors, cursor sync',
      'Document sync: broadcast graph state to all peers',
      'Health check endpoint at GET /health',
      '12 Gleam tests',
    ],
  },
  {
    name: 'Deployment',
    tag: 'Infrastructure',
    tech: 'Docker, docker-compose, GitHub Actions CI',
    desc: 'Containerised deployment with health checks and CI pipeline.',
    points: [
      'Multi-stage web Dockerfile (build → preview)',
      'Server Dockerfile with gleam build and health check',
      'docker-compose with depends_on and service_healthy',
      'CI: Rust (fmt, clippy, test), Gleam (build, test), Web (vue-tsc, build)',
    ],
  },
]

const stats = [
  { label: 'Rust tests', value: '146' },
  { label: 'Gleam tests', value: '12' },
  { label: 'Node types', value: '3' },
  { label: 'Preset scenarios', value: '9' },
  { label: 'Max nodes (tested)', value: '120+' },
  { label: 'Build days', value: '30' },
]
</script>

<template>
  <div class="arch">
    <section class="fl-section">
      <div class="fl-container">
        <p class="arch__kicker">Architecture</p>
        <h1 class="arch__title">How FaultLab is built</h1>
        <p class="arch__lede">
          Six layers, from browser to server. Each layer is independently
          testable and replaceable.
        </p>

        <div class="arch__stats">
          <div v-for="s in stats" :key="s.label" class="arch__stat">
            <span class="arch__stat-value">{{ s.value }}</span>
            <span class="arch__stat-label">{{ s.label }}</span>
          </div>
        </div>
      </div>
    </section>

    <section class="fl-section fl-section--alt">
      <div class="fl-container">
        <div class="arch__layers">
          <div
            v-for="(layer, i) in layers"
            :key="layer.name"
            class="arch__layer"
          >
            <div class="arch__layer-number">{{ i + 1 }}</div>
            <div class="arch__layer-content">
              <div class="arch__layer-header">
                <span class="fl-tag fl-tag--blue">{{ layer.tag }}</span>
                <h2>{{ layer.name }}</h2>
              </div>
              <p class="arch__layer-tech">{{ layer.tech }}</p>
              <p class="arch__layer-desc">{{ layer.desc }}</p>
              <ul class="arch__layer-points">
                <li v-for="p in layer.points" :key="p">{{ p }}</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section class="fl-section">
      <div class="fl-container arch__cta">
        <h2>See it in action</h2>
        <RouterLink to="/editor" class="fl-button fl-button--primary">
          Open the editor
        </RouterLink>
      </div>
    </section>
  </div>
</template>

<style scoped>
.arch__kicker {
  font-size: var(--fl-size-19);
  font-weight: 700;
  color: var(--fl-amber-hover);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: var(--fl-space-2);
}

.arch__title {
  font-size: var(--fl-size-48);
  margin-bottom: var(--fl-space-3);
}

.arch__lede {
  font-size: var(--fl-size-24);
  color: var(--fl-grey-4);
  max-width: 640px;
  margin-bottom: var(--fl-space-6);
}

.arch__stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: var(--fl-space-3);
}

.arch__stat {
  text-align: center;
  background: var(--fl-white);
  border: 1px solid var(--fl-border);
  border-top: 3px solid var(--fl-amber);
  padding: var(--fl-space-3);
  box-shadow: var(--fl-shadow-sm);
  transition: box-shadow var(--fl-transition), transform var(--fl-transition);
}

.arch__stat:hover {
  box-shadow: var(--fl-shadow-md);
  transform: translateY(-2px);
}

.arch__stat-value {
  display: block;
  font-size: var(--fl-size-36);
  font-weight: 800;
  color: var(--fl-slate);
}

.arch__stat-label {
  font-size: var(--fl-size-14);
  color: var(--fl-grey-4);
}

.arch__layers {
  display: flex;
  flex-direction: column;
  gap: var(--fl-space-4);
}

.arch__layer {
  display: flex;
  gap: var(--fl-space-4);
  background: var(--fl-white);
  border: 1px solid var(--fl-border);
  border-left: 3px solid var(--fl-slate);
  padding: var(--fl-space-4);
  box-shadow: var(--fl-shadow-sm);
  transition: box-shadow var(--fl-transition);
}

.arch__layer:hover {
  box-shadow: var(--fl-shadow-md);
}

.arch__layer-number {
  font-size: var(--fl-size-36);
  font-weight: 800;
  color: var(--fl-amber);
  flex-shrink: 0;
  width: 48px;
  text-align: center;
}

.arch__layer-content {
  flex: 1;
}

.arch__layer-header {
  display: flex;
  align-items: center;
  gap: var(--fl-space-2);
  margin-bottom: var(--fl-space-1);
}

.arch__layer-header h2 {
  font-size: var(--fl-size-24);
}

.arch__layer-tech {
  font-size: var(--fl-size-14);
  font-weight: 600;
  color: var(--fl-grey-3);
  margin-bottom: var(--fl-space-2);
}

.arch__layer-desc {
  font-size: var(--fl-size-19);
  color: var(--fl-grey-4);
  margin-bottom: var(--fl-space-3);
}

.arch__layer-points {
  list-style: disc;
  padding-left: var(--fl-space-5);
}

.arch__layer-points li {
  font-size: var(--fl-size-16);
  color: var(--fl-grey-4);
  margin-bottom: var(--fl-space-1);
}

.arch__cta {
  text-align: center;
  padding: var(--fl-space-6);
}

.arch__cta h2 {
  font-size: var(--fl-size-36);
  margin-bottom: var(--fl-space-4);
}

@media (max-width: 768px) {
  .arch__layer {
    flex-direction: column;
  }

  .arch__layer-number {
    width: auto;
    text-align: left;
  }
}
</style>
