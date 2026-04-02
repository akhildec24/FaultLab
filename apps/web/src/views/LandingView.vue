<script setup lang="ts">
import { RouterLink } from 'vue-router'
import { PRESETS } from '@/graph/presets'

const components = [
  { name: 'Client', desc: 'Generates traffic at a configurable rate.', tag: 'Source', icon: 'C' },
  { name: 'Service', desc: 'Processes requests with capacity and latency limits.', tag: 'Compute', icon: 'S' },
  { name: 'Queue', desc: 'Buffers messages and overflows when full.', tag: 'Buffer', icon: 'Q' },
  { name: 'Cache', desc: 'Returns cached responses or forwards misses.', tag: 'Storage', icon: '$' },
  { name: 'Database', desc: 'Stores data with query capacity and latency.', tag: 'Storage', icon: 'D' },
  { name: 'External API', desc: 'Third-party service outside your control.', tag: 'External', icon: 'E' },
]

const failures = [
  { text: 'Take a server offline', icon: 'x' },
  { text: 'Add 500ms of network latency', icon: '~' },
  { text: 'Disconnect the database', icon: '/' },
  { text: 'Cause packet loss', icon: '%' },
  { text: 'Fill the message queue', icon: '#' },
  { text: 'Slow an external API', icon: 's' },
  { text: 'Create a network partition', icon: '|' },
]

const metrics = [
  { label: 'Requests per second', value: 'Live throughput' },
  { label: 'Success / failure rate', value: 'Outcome tracking' },
  { label: 'Response-time percentiles', value: 'P50, P95, P99' },
  { label: 'Queue depth', value: 'Backlog visibility' },
  { label: 'Retry storms', value: 'Cascade detection' },
  { label: 'Component utilisation', value: 'Saturation monitoring' },
]

const scenarioColors: Record<string, string> = {
  'overloaded-database': '#ef4444',
  'cascading-failure': '#dc2626',
  'network-partition': '#f97316',
  'retry-storm': '#ef4444',
  'queue-overflow': '#f59e0b',
  'cache-replication': '#3b82f6',
  'replication-delay': '#8b5cf6',
  'microservice-mesh': '#06b6d4',
  'thundering-herd': '#ec4899',
}
</script>

<template>
  <div class="landing">
    <!-- Hero -->
    <section class="landing__hero">
      <div class="fl-container landing__hero-inner">
        <div class="landing__hero-content">
          <p class="landing__kicker">Distributed systems simulator</p>
          <h1 class="landing__title">Break systems on purpose.<br />Learn what happens.</h1>
          <p class="landing__lede">
            Design a software architecture, send simulated traffic through it,
            inject failures, and watch the consequences unfold — all in your
            browser.
          </p>
          <div class="landing__actions">
            <RouterLink to="/editor" class="fl-button fl-button--primary">
              Start building
            </RouterLink>
            <RouterLink to="/architecture" class="fl-button fl-button--secondary">
              Architecture
            </RouterLink>
            <RouterLink to="/docs" class="fl-button fl-button--secondary">
              Read the docs
            </RouterLink>
          </div>
        </div>
        <div class="landing__hero-visual">
          <div class="landing__hero-diagram">
            <div class="landing__hero-node landing__hero-node--client">
              <span class="landing__hero-node-icon">C</span>
              <span class="landing__hero-node-label">Client</span>
            </div>
            <div class="landing__hero-line" />
            <div class="landing__hero-node landing__hero-node--service">
              <span class="landing__hero-node-icon">S</span>
              <span class="landing__hero-node-label">Service</span>
            </div>
            <div class="landing__hero-line" />
            <div class="landing__hero-node landing__hero-node--database">
              <span class="landing__hero-node-icon">D</span>
              <span class="landing__hero-node-label">Database</span>
            </div>
            <div class="landing__hero-pulse" />
          </div>
        </div>
      </div>
    </section>

    <!-- What it does -->
    <section class="fl-section fl-section--alt">
      <div class="fl-container">
        <h2 class="landing__heading">How it works</h2>
        <div class="fl-grid fl-grid--3">
          <div class="landing__card">
            <div class="landing__card-number">1</div>
            <h3>Design</h3>
            <p>Drag components onto a canvas. Connect them. Configure
            capacity, latency, error rates, and retry policies.</p>
          </div>
          <div class="landing__card">
            <div class="landing__card-number">2</div>
            <h3>Run</h3>
            <p>Send traffic through the system. Watch requests move in real
            time. See metrics update as load increases.</p>
          </div>
          <div class="landing__card">
            <div class="landing__card-number">3</div>
            <h3>Break</h3>
            <p>Inject failures — crash a server, add latency, disconnect a
            database. Observe how the architecture degrades.</p>
          </div>
        </div>
      </div>
    </section>

    <!-- Components -->
    <section class="fl-section">
      <div class="fl-container">
        <h2 class="landing__heading">Available components</h2>
        <p class="landing__subheading">
          Six component types to start with. More arrive as the project grows.
        </p>
        <div class="fl-grid fl-grid--3">
          <div v-for="c in components" :key="c.name" class="landing__component">
            <div class="landing__component-icon">{{ c.icon }}</div>
            <div class="landing__component-body">
              <div class="landing__component-header">
                <span class="fl-tag fl-tag--green">{{ c.tag }}</span>
                <h3>{{ c.name }}</h3>
              </div>
              <p>{{ c.desc }}</p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Failures -->
    <section class="fl-section fl-section--alt">
      <div class="fl-container">
        <h2 class="landing__heading">Inject failures</h2>
        <p class="landing__subheading">
          Trigger incidents mid-simulation and watch the system react.
        </p>
        <div class="landing__failures">
          <div v-for="f in failures" :key="f.text" class="landing__failure">
            <span class="landing__failure-icon">{{ f.icon }}</span>
            <span>{{ f.text }}</span>
          </div>
        </div>
      </div>
    </section>

    <!-- Metrics -->
    <section class="fl-section">
      <div class="fl-container">
        <h2 class="landing__heading">What you observe</h2>
        <div class="fl-grid fl-grid--3">
          <div v-for="m in metrics" :key="m.label" class="landing__metric">
            <h3 class="landing__metric-label">{{ m.label }}</h3>
            <p class="landing__metric-value">{{ m.value }}</p>
          </div>
        </div>
      </div>
    </section>

    <!-- Preset scenarios -->
    <section class="fl-section fl-section--alt">
      <div class="fl-container">
        <h2 class="landing__heading">Preset scenarios</h2>
        <p class="landing__subheading">
          {{ PRESETS.length }} ready-to-run scenarios. Load one in the editor and press Run.
        </p>
        <div class="landing__presets">
          <RouterLink
            v-for="p in PRESETS"
            :key="p.id"
            :to="{ path: '/editor', query: { preset: p.id } }"
            class="landing__preset"
          >
            <div class="landing__preset-bar" :style="{ background: scenarioColors[p.id] || '#f59e0b' }" />
            <h3>{{ p.name }}</h3>
            <p>{{ p.description }}</p>
            <span class="landing__preset-cta">Open in editor →</span>
          </RouterLink>
        </div>
      </div>
    </section>

    <!-- Architecture -->
    <section class="fl-section">
      <div class="fl-container">
        <h2 class="landing__heading">Under the hood</h2>
        <p class="landing__subheading">
          The simulation runs in a deterministic engine — not JavaScript timers.
        </p>
        <div class="landing__stack">
          <div class="landing__stack-layer">
            <span class="fl-tag fl-tag--blue">Interface</span>
            Vue 3 + TypeScript
          </div>
          <div class="landing__stack-arrow">↓</div>
          <div class="landing__stack-layer">
            <span class="fl-tag fl-tag--blue">Worker</span>
            Web Worker (off-main-thread)
          </div>
          <div class="landing__stack-arrow">↓</div>
          <div class="landing__stack-layer">
            <span class="fl-tag fl-tag--green">Engine</span>
            Rust compiled to WebAssembly
          </div>
        </div>
        <div class="landing__stack landing__stack--collab">
          <div class="landing__stack-layer">
            <span class="fl-tag fl-tag--yellow">Document</span>
            Automerge CRDT (local-first)
          </div>
          <div class="landing__stack-arrow">↓</div>
          <div class="landing__stack-layer">
            <span class="fl-tag fl-tag--yellow">Sync</span>
            WebSocket
          </div>
          <div class="landing__stack-arrow">↓</div>
          <div class="landing__stack-layer">
            <span class="fl-tag fl-tag--yellow">Server</span>
            Gleam + Erlang/OTP
          </div>
        </div>
      </div>
    </section>

    <!-- CTA -->
    <section class="fl-section fl-section--alt landing__cta">
      <div class="fl-container landing__cta-inner">
        <h2 class="landing__heading">Ready to break something?</h2>
        <p class="landing__subheading">
          No account needed. Runs entirely in your browser.
        </p>
        <RouterLink to="/editor" class="fl-button fl-button--primary">
          Open the editor
        </RouterLink>
      </div>
    </section>
  </div>
</template>

<style scoped>
/* Hero */
.landing__hero {
  padding: var(--fl-space-7) 0;
  border-bottom: 1px solid var(--fl-border);
  background: linear-gradient(180deg, var(--fl-bg-alt) 0%, var(--fl-white) 100%);
}

.landing__hero-inner {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--fl-space-6);
  align-items: center;
}

.landing__hero-content {
  max-width: 540px;
}

.landing__kicker {
  font-size: var(--fl-size-19);
  font-weight: 700;
  color: var(--fl-amber-hover);
  margin-bottom: var(--fl-space-2);
  letter-spacing: 0.5px;
  text-transform: uppercase;
}

.landing__title {
  font-size: var(--fl-size-48);
  line-height: var(--fl-leading-tight);
  margin-bottom: var(--fl-space-4);
}

.landing__lede {
  font-size: var(--fl-size-24);
  line-height: var(--fl-leading-normal);
  color: var(--fl-grey-4);
  margin-bottom: var(--fl-space-5);
}

.landing__actions {
  display: flex;
  gap: var(--fl-space-3);
  flex-wrap: wrap;
}

/* Hero visual */
.landing__hero-visual {
  display: flex;
  align-items: center;
  justify-content: center;
}

.landing__hero-diagram {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fl-space-3);
}

.landing__hero-node {
  display: flex;
  align-items: center;
  gap: var(--fl-space-2);
  width: 200px;
  padding: var(--fl-space-2) var(--fl-space-3);
  border: 2px solid var(--fl-slate);
  background: var(--fl-white);
  box-shadow: var(--fl-shadow-md);
  transition: transform var(--fl-transition), box-shadow var(--fl-transition);
}

.landing__hero-node:hover {
  transform: translateX(4px);
  box-shadow: var(--fl-shadow-lg);
}

.landing__hero-node--client {
  border-color: var(--fl-amber);
}

.landing__hero-node--service {
  border-color: #6366f1;
}

.landing__hero-node--database {
  border-color: var(--fl-green);
}

.landing__hero-node-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  color: var(--fl-white);
  font-weight: 700;
  font-size: var(--fl-size-16);
  flex-shrink: 0;
}

.landing__hero-node--client .landing__hero-node-icon {
  background: var(--fl-amber);
}

.landing__hero-node--service .landing__hero-node-icon {
  background: #6366f1;
}

.landing__hero-node--database .landing__hero-node-icon {
  background: var(--fl-green);
}

.landing__hero-node-label {
  font-weight: 700;
  font-size: var(--fl-size-16);
  color: var(--fl-text);
}

.landing__hero-line {
  width: 2px;
  height: 24px;
  background: var(--fl-grey-2);
}

.landing__hero-pulse {
  position: absolute;
  left: 16px;
  top: 30px;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--fl-amber);
  animation: pulse-down 2s ease-in-out infinite;
}

@keyframes pulse-down {
  0% { transform: translateY(0); opacity: 1; }
  100% { transform: translateY(180px); opacity: 0; }
}

/* Headings */
.landing__heading {
  font-size: var(--fl-size-36);
  margin-bottom: var(--fl-space-3);
}

.landing__subheading {
  font-size: var(--fl-size-19);
  color: var(--fl-grey-4);
  margin-bottom: var(--fl-space-5);
  max-width: 640px;
}

/* Cards */
.landing__card {
  background: var(--fl-white);
  border: 1px solid var(--fl-border);
  border-top: 3px solid var(--fl-amber);
  padding: var(--fl-space-4);
  box-shadow: var(--fl-shadow-sm);
  transition: box-shadow var(--fl-transition), transform var(--fl-transition);
}

.landing__card:hover {
  box-shadow: var(--fl-shadow-md);
  transform: translateY(-2px);
}

.landing__card-number {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: var(--fl-slate);
  color: var(--fl-amber);
  font-size: var(--fl-size-19);
  font-weight: 800;
  margin-bottom: var(--fl-space-2);
}

.landing__card h3 {
  font-size: var(--fl-size-24);
  margin-bottom: var(--fl-space-2);
}

.landing__card p {
  font-size: var(--fl-size-19);
  color: var(--fl-grey-4);
}

/* Components */
.landing__component {
  display: flex;
  gap: var(--fl-space-3);
  background: var(--fl-white);
  border: 1px solid var(--fl-border);
  border-left: 3px solid var(--fl-slate);
  padding: var(--fl-space-3);
  box-shadow: var(--fl-shadow-sm);
  transition: box-shadow var(--fl-transition);
}

.landing__component:hover {
  box-shadow: var(--fl-shadow-md);
}

.landing__component-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 6px;
  background: var(--fl-slate);
  color: var(--fl-amber);
  font-weight: 800;
  font-size: var(--fl-size-19);
  flex-shrink: 0;
}

.landing__component-header {
  display: flex;
  align-items: center;
  gap: var(--fl-space-2);
  margin-bottom: var(--fl-space-1);
}

.landing__component-header h3 {
  font-size: var(--fl-size-19);
}

.landing__component p {
  font-size: var(--fl-size-16);
  color: var(--fl-grey-4);
}

/* Failures */
.landing__failures {
  display: flex;
  flex-wrap: wrap;
  gap: var(--fl-space-2);
  max-width: 720px;
}

.landing__failure {
  display: flex;
  align-items: center;
  gap: var(--fl-space-2);
  padding: var(--fl-space-1) var(--fl-space-3);
  background: var(--fl-white);
  border: 1px solid var(--fl-border);
  border-left: 3px solid var(--fl-red);
  font-size: var(--fl-size-16);
  color: var(--fl-text);
  box-shadow: var(--fl-shadow-sm);
  transition: border-color var(--fl-transition), box-shadow var(--fl-transition);
}

.landing__failure:hover {
  border-left-color: var(--fl-amber);
  box-shadow: var(--fl-shadow-md);
}

.landing__failure-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 4px;
  background: var(--fl-red-light);
  color: var(--fl-red);
  font-family: var(--fl-font-mono);
  font-weight: 700;
  font-size: 0.8rem;
  flex-shrink: 0;
}

/* Metrics */
.landing__metric {
  border-left: 3px solid var(--fl-amber);
  padding-left: var(--fl-space-4);
}

.landing__metric-label {
  font-size: var(--fl-size-19);
  font-weight: 700;
}

.landing__metric-value {
  font-size: var(--fl-size-16);
  color: var(--fl-grey-4);
}

/* Presets */
.landing__presets {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--fl-space-3);
}

.landing__preset {
  display: flex;
  flex-direction: column;
  background: var(--fl-white);
  border: 1px solid var(--fl-border);
  padding: var(--fl-space-4);
  text-decoration: none;
  color: inherit;
  box-shadow: var(--fl-shadow-sm);
  transition: box-shadow var(--fl-transition), transform var(--fl-transition);
}

.landing__preset:hover {
  box-shadow: var(--fl-shadow-md);
  transform: translateY(-2px);
}

.landing__preset-bar {
  height: 4px;
  width: 100%;
  margin-bottom: var(--fl-space-3);
}

.landing__preset h3 {
  font-size: var(--fl-size-19);
  margin-bottom: var(--fl-space-2);
}

.landing__preset p {
  font-size: var(--fl-size-14);
  color: var(--fl-grey-4);
  margin-bottom: var(--fl-space-3);
  flex: 1;
}

.landing__preset-cta {
  font-size: var(--fl-size-14);
  font-weight: 700;
  color: var(--fl-amber-hover);
}

/* Stack */
.landing__stack {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fl-space-2);
  max-width: 400px;
  margin-bottom: var(--fl-space-6);
}

.landing__stack--collab {
  margin-bottom: 0;
}

.landing__stack-layer {
  display: flex;
  align-items: center;
  gap: var(--fl-space-2);
  width: 100%;
  background: var(--fl-white);
  border: 1px solid var(--fl-border);
  border-left: 3px solid var(--fl-slate);
  padding: var(--fl-space-3) var(--fl-space-4);
  font-size: var(--fl-size-19);
  font-weight: 700;
  box-shadow: var(--fl-shadow-sm);
}

.landing__stack-arrow {
  font-size: var(--fl-size-19);
  color: var(--fl-grey-3);
}

/* CTA */
.landing__cta-inner {
  text-align: center;
  padding: var(--fl-space-6) var(--fl-space-4);
}

@media (max-width: 768px) {
  .landing__hero-inner {
    grid-template-columns: 1fr;
  }

  .landing__hero-visual {
    display: none;
  }

  .landing__title {
    font-size: var(--fl-size-36);
  }

  .landing__lede {
    font-size: var(--fl-size-19);
  }
}
</style>
