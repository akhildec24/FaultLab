<script setup lang="ts">
import { RouterLink } from 'vue-router'

const guides = [
  {
    title: 'Getting Started',
    desc: 'Load a preset, run a simulation, and read the results in under 2 minutes.',
    steps: [
      'Open the Editor and pick a preset from the dropdown',
      'Click Run to start the simulation — traffic flows through your topology',
      'Watch the status bar for live clock, pending events, and metrics',
      'Expand the Timeline panel to see every event as it fires',
      'Use Step to advance one event at a time for detailed analysis',
    ],
  },
  {
    title: 'Building Your Own Topology',
    desc: 'Add nodes, connect them, and configure each component.',
    steps: [
      'Use the toolbar buttons to add Clients, Services, Databases, Queues, Caches, and External APIs',
      'Click a node to select it — the inspector panel shows all properties',
      'Drag the amber handle on any node to draw a connection to another',
      'Adjust capacity, latency, error rate, timeout, retry policy, and queue limits per node',
      'Click Run when your graph is ready — validation warnings will show if something is missing',
    ],
  },
  {
    title: 'Injecting Failures',
    desc: 'Break things mid-simulation to see how your system degrades.',
    steps: [
      'Start a simulation with any preset or custom topology',
      'Use the Failure panel next to the metrics to choose a failure type',
      'Crash a node to take it offline, or Recover to bring it back',
      'Add latency to a node to simulate slow responses',
      'Disconnect a link to simulate a network partition',
      'Reduce capacity to stress-test a bottleneck',
    ],
  },
  {
    title: 'Importing & Exporting',
    desc: 'Save your work, share scenarios, and import existing topologies.',
    steps: [
      'Click the Storage button in the toolbar to open the storage panel',
      'Use Export to download your current topology as a JSON file',
      'Use Import to load a previously exported FaultLab scenario',
      'Scenarios auto-save to your browser via IndexedDB',
      'Take manual snapshots to capture and restore points in time',
    ],
  },
  {
    title: 'Reading the Timeline',
    desc: 'Understand what happens inside your system, event by event.',
    steps: [
      'Expand the bottom panel and switch to the Timeline tab',
      'Each row shows the virtual time, event type, and affected node',
      'Use the filter boxes to narrow by node ID, event type, or request ID',
      'Click Step to advance one event and watch the cascade unfold',
      'Colour-coded badges show success (green), failure (red), and timeout (amber)',
    ],
  },
  {
    title: 'Keyboard Shortcuts',
    desc: 'Move faster in the editor with these shortcuts.',
    steps: [
      'Hold Space — grab cursor, drag to pan the canvas from anywhere',
      'Scroll — zoom in and out toward the cursor',
      'Delete / Backspace — remove the selected node or edge',
      'Escape — cancel connection mode or clear selection',
      'Click amber handle — start a connection from that node',
    ],
  },
]

const nodeTypes = [
  { name: 'Client', icon: 'C', color: '#f59e0b', shape: 'Pill', desc: 'Generates traffic at a configurable rate. Every simulation needs at least one.' },
  { name: 'Service', icon: 'S', color: '#6366f1', shape: 'Rounded rect', desc: 'Processes requests with capacity, latency, and retry policies. The workhorse.' },
  { name: 'Database', icon: 'D', color: '#059669', shape: 'Cylinder', desc: 'Stores data with configurable query capacity and latency. Supports leader/replica.' },
  { name: 'Queue', icon: 'Q', color: '#ec4899', shape: 'Parallelogram', desc: 'Buffers messages between services. Overflows when the queue limit is reached.' },
  { name: 'Cache', icon: '$', color: '#ea580c', shape: 'Hexagon', desc: 'Fast lookup layer with low latency and high capacity. Misses forward to origin.' },
  { name: 'External API', icon: 'E', color: '#64748b', shape: 'Cloud', desc: 'Third-party service outside your control. High latency and error rate by default.' },
]

const presetCount = 15
</script>

<template>
  <div class="fl-container fl-section">
    <h1>Documentation</h1>
    <p class="docs__lede">
      Learn how to use FaultLab to simulate distributed systems, inject
      failures, and understand how your architecture behaves under stress.
    </p>

    <!-- Quick start -->
    <h2 class="docs__heading">Guides</h2>
    <div class="fl-grid fl-grid--2">
      <div
        v-for="g in guides"
        :key="g.title"
        class="docs__card"
      >
        <h3>{{ g.title }}</h3>
        <p class="docs__card-desc">{{ g.desc }}</p>
        <ol class="docs__steps">
          <li v-for="(step, i) in g.steps" :key="i">{{ step }}</li>
        </ol>
      </div>
    </div>

    <!-- Node types -->
    <h2 class="docs__heading">Node Types</h2>
    <p class="docs__progress">
      FaultLab supports {{ nodeTypes.length }} node types, each with a distinct
      shape and colour. Combine them to model any distributed system.
    </p>
    <div class="docs__nodes">
      <div v-for="n in nodeTypes" :key="n.name" class="docs__node">
        <span class="docs__node-icon" :style="{ background: n.color }">{{ n.icon }}</span>
        <div class="docs__node-body">
          <div class="docs__node-header">
            <span class="docs__node-name">{{ n.name }}</span>
            <span class="docs__node-shape">{{ n.shape }}</span>
          </div>
          <p class="docs__node-desc">{{ n.desc }}</p>
        </div>
      </div>
    </div>

    <!-- Presets -->
    <h2 class="docs__heading">Preset Scenarios</h2>
    <p class="docs__progress">
      FaultLab ships with {{ presetCount }} ready-to-run preset scenarios
      modelled on real-world systems — from e-commerce checkouts to IoT
      telemetry pipelines. Open the
      <RouterLink to="/editor">editor</RouterLink> to try them.
    </p>

    <!-- Tech stack (kept for reference) -->
    <h2 class="docs__heading">Under the Hood</h2>
    <table class="docs__table">
      <thead>
        <tr>
          <th>Layer</th>
          <th>Technology</th>
        </tr>
      </thead>
      <tbody>
        <tr><td>Interface</td><td>Vue 3, TypeScript, Vite, Pinia</td></tr>
        <tr><td>Simulation engine</td><td>Rust, wasm-bindgen, serde</td></tr>
        <tr><td>Collaboration</td><td>Gleam, Erlang/OTP, WebSockets</td></tr>
        <tr><td>Local-first storage</td><td>IndexedDB, auto-save, snapshots</td></tr>
        <tr><td>Deployment</td><td>Docker, Docker Compose, GitHub Actions</td></tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.docs__lede {
  font-size: var(--fl-size-19);
  color: var(--fl-grey-4);
  max-width: 640px;
  margin-bottom: var(--fl-space-5);
}

.docs__heading {
  font-size: var(--fl-size-27);
  margin: var(--fl-space-6) 0 var(--fl-space-4);
}

.docs__card {
  display: flex;
  flex-direction: column;
  background: var(--fl-bg-alt);
  border-left: 3px solid var(--fl-amber);
  padding: var(--fl-space-4);
  text-decoration: none;
  color: inherit;
  box-shadow: var(--fl-shadow-sm);
  transition: box-shadow var(--fl-transition), transform var(--fl-transition);
}

.docs__card:hover {
  box-shadow: var(--fl-shadow-md);
  transform: translateY(-2px);
}

.docs__card h3 {
  font-size: var(--fl-size-19);
  margin-bottom: var(--fl-space-2);
}

.docs__card p {
  font-size: var(--fl-size-16);
  color: var(--fl-grey-4);
  flex: 1;
}

.docs__card-desc {
  margin-bottom: var(--fl-space-3);
}

.docs__steps {
  margin: 0;
  padding-left: var(--fl-space-4);
  font-size: var(--fl-size-16);
  color: var(--fl-grey-4);
  line-height: 1.6;
}

.docs__steps li {
  margin-bottom: var(--fl-space-1);
}

.docs__nodes {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: var(--fl-space-3);
  margin-bottom: var(--fl-space-5);
}

.docs__node {
  display: flex;
  gap: var(--fl-space-3);
  background: var(--fl-bg-alt);
  padding: var(--fl-space-3);
  border-left: 3px solid var(--fl-amber);
}

.docs__node-icon {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 14px;
}

.docs__node-body {
  flex: 1;
}

.docs__node-header {
  display: flex;
  align-items: center;
  gap: var(--fl-space-2);
  margin-bottom: var(--fl-space-1);
}

.docs__node-name {
  font-weight: 700;
  font-size: var(--fl-size-16);
}

.docs__node-shape {
  font-size: var(--fl-size-14);
  color: var(--fl-grey-3);
  background: var(--fl-bg);
  padding: 2px 8px;
  border-radius: 4px;
}

.docs__node-desc {
  font-size: var(--fl-size-14);
  color: var(--fl-grey-4);
  line-height: 1.5;
}

.docs__table {
  width: 100%;
  border-collapse: collapse;
}

.docs__table th,
.docs__table td {
  text-align: left;
  padding: var(--fl-space-2) var(--fl-space-3);
  border-bottom: 1px solid var(--fl-border);
  font-size: var(--fl-size-19);
}

.docs__table th {
  font-weight: 700;
  background: var(--fl-bg-alt);
}

.docs__progress {
  font-size: var(--fl-size-19);
  color: var(--fl-grey-4);
  max-width: 640px;
}
</style>
