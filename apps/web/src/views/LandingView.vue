<script setup lang="ts">
import { RouterLink } from 'vue-router'

const components = [
  { name: 'Client', desc: 'Generates traffic at a configurable rate.', tag: 'Source' },
  { name: 'Service', desc: 'Processes requests with capacity and latency limits.', tag: 'Compute' },
  { name: 'Queue', desc: 'Buffers messages and overflows when full.', tag: 'Buffer' },
  { name: 'Cache', desc: 'Returns cached responses or forwards misses.', tag: 'Storage' },
  { name: 'Database', desc: 'Stores data with query capacity and latency.', tag: 'Storage' },
  { name: 'External API', desc: 'Third-party service outside your control.', tag: 'External' },
]

const failures = [
  'Take a server offline',
  'Add 500ms of network latency',
  'Disconnect the database',
  'Cause packet loss',
  'Fill the message queue',
  'Slow an external API',
  'Create a network partition',
]

const metrics = [
  { label: 'Requests per second', value: 'Live throughput' },
  { label: 'Success / failure rate', value: 'Outcome tracking' },
  { label: 'Response-time percentiles', value: 'P50, P95, P99' },
  { label: 'Queue depth', value: 'Backlog visibility' },
  { label: 'Retry storms', value: 'Cascade detection' },
  { label: 'Component utilisation', value: 'Saturation monitoring' },
]
</script>

<template>
  <div class="landing">
    <!-- Hero -->
    <section class="fl-section landing__hero">
      <div class="fl-container">
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
          <RouterLink to="/docs" class="fl-button fl-button--secondary">
            Read the docs
          </RouterLink>
        </div>
      </div>
    </section>

    <!-- What it does -->
    <section class="fl-section fl-section--alt">
      <div class="fl-container">
        <h2 class="landing__heading">How it works</h2>
        <div class="fl-grid fl-grid--3">
          <div class="landing__card">
            <span class="fl-tag fl-tag--blue">1</span>
            <h3>Design</h3>
            <p>Drag components onto a canvas. Connect them. Configure
            capacity, latency, error rates, and retry policies.</p>
          </div>
          <div class="landing__card">
            <span class="fl-tag fl-tag--blue">2</span>
            <h3>Run</h3>
            <p>Send traffic through the system. Watch requests move in real
            time. See metrics update as load increases.</p>
          </div>
          <div class="landing__card">
            <span class="fl-tag fl-tag--blue">3</span>
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
            <div class="landing__component-header">
              <span class="fl-tag fl-tag--green">{{ c.tag }}</span>
              <h3>{{ c.name }}</h3>
            </div>
            <p>{{ c.desc }}</p>
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
        <ul class="fl-list fl-list--bullet landing__failure-list">
          <li v-for="f in failures" :key="f">{{ f }}</li>
        </ul>
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

    <!-- Demo scenario -->
    <section class="fl-section fl-section--alt">
      <div class="fl-container">
        <h2 class="landing__heading">Example: the retry storm</h2>
        <div class="landing__demo">
          <div class="landing__demo-diagram">
            <div class="landing__demo-node">Customer</div>
            <div class="landing__demo-arrow">↓</div>
            <div class="landing__demo-node">Load Balancer</div>
            <div class="landing__demo-arrow">↓</div>
            <div class="landing__demo-node">API Servers</div>
            <div class="landing__demo-arrow">↓</div>
            <div class="landing__demo-node">Payment Service</div>
            <div class="landing__demo-arrow">↓</div>
            <div class="landing__demo-node landing__demo-node--danger">Database</div>
          </div>
          <div class="landing__demo-text">
            <p>Traffic ramps from 50 to 1,000 requests per second. The payment
            service begins timing out. The API retries every failed request
            immediately.</p>
            <p>Retries create more traffic. The database is overwhelmed. A
            retry storm cascades through the system.</p>
            <p>Enable exponential backoff, a circuit breaker, and a max retry
            limit. Run the same simulation. Compare the results.</p>
            <RouterLink to="/editor" class="fl-button fl-button--primary">
              Try this scenario
            </RouterLink>
          </div>
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
.landing__hero {
  padding: var(--fl-space-7) 0;
  border-bottom: 1px solid var(--fl-border);
  background: linear-gradient(180deg, var(--fl-bg-alt) 0%, var(--fl-white) 100%);
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
  max-width: 640px;
  margin-bottom: var(--fl-space-5);
}

.landing__actions {
  display: flex;
  gap: var(--fl-space-3);
  flex-wrap: wrap;
}

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

.landing__card {
  background: var(--fl-white);
  border: 1px solid var(--fl-border);
  border-top: 3px solid var(--fl-amber);
  padding: var(--fl-space-4);
}

.landing__card h3 {
  font-size: var(--fl-size-24);
  margin: var(--fl-space-2) 0 var(--fl-space-2);
}

.landing__card p {
  font-size: var(--fl-size-19);
  color: var(--fl-grey-4);
}

.landing__component {
  background: var(--fl-white);
  border: 1px solid var(--fl-border);
  border-left: 3px solid var(--fl-slate);
  padding: var(--fl-space-4);
}

.landing__component-header {
  display: flex;
  align-items: center;
  gap: var(--fl-space-2);
  margin-bottom: var(--fl-space-2);
}

.landing__component-header h3 {
  font-size: var(--fl-size-24);
}

.landing__component p {
  font-size: var(--fl-size-19);
  color: var(--fl-grey-4);
}

.landing__failure-list {
  max-width: 480px;
}

.landing__failure-list li {
  font-size: var(--fl-size-19);
  margin-bottom: var(--fl-space-2);
}

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

.landing__demo {
  display: grid;
  grid-template-columns: 280px 1fr;
  gap: var(--fl-space-6);
  align-items: start;
}

.landing__demo-diagram {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fl-space-1);
}

.landing__demo-node {
  width: 100%;
  text-align: center;
  background: var(--fl-white);
  border: 2px solid var(--fl-slate);
  padding: var(--fl-space-2) var(--fl-space-3);
  font-weight: 700;
  font-size: var(--fl-size-16);
}

.landing__demo-node--danger {
  border-color: var(--fl-red);
  color: var(--fl-red);
  background: var(--fl-red-light);
}

.landing__demo-arrow {
  font-size: var(--fl-size-19);
  color: var(--fl-grey-3);
}

.landing__demo-text p {
  margin-bottom: var(--fl-space-3);
  font-size: var(--fl-size-19);
}

.landing__demo-text .fl-button {
  margin-top: var(--fl-space-2);
}

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
}

.landing__stack-arrow {
  font-size: var(--fl-size-19);
  color: var(--fl-grey-3);
}

.landing__cta-inner {
  text-align: center;
  padding: var(--fl-space-6) var(--fl-space-4);
}

@media (max-width: 768px) {
  .landing__demo {
    grid-template-columns: 1fr;
  }

  .landing__title {
    font-size: var(--fl-size-36);
  }

  .landing__lede {
    font-size: var(--fl-size-19);
  }
}
</style>
