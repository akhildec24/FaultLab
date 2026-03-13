<script setup lang="ts">
const sections = [
  {
    title: 'Product specification',
    desc: 'What FaultLab is, who it serves, and what the first version includes.',
    href: '/docs/SPEC.md',
  },
  {
    title: '30-day build plan',
    desc: 'Day-by-day breakdown of the entire build, week by week.',
    href: '/docs/PLAN.md',
  },
  {
    title: 'Architecture decisions',
    desc: 'Records of key technical decisions and their trade-offs.',
    href: '/docs/decisions/',
  },
]

const stack = [
  { layer: 'Interface', tech: 'Vue 3, TypeScript, Vite, Pinia' },
  { layer: 'Simulation engine', tech: 'Rust, wasm-bindgen, serde' },
  { layer: 'Collaboration', tech: 'Gleam, Erlang/OTP, WebSockets' },
  { layer: 'Local-first', tech: 'Automerge, IndexedDB' },
  { layer: 'Scenario language', tech: 'Custom DSL (lexer, parser, AST)' },
  { layer: 'Deployment', tech: 'Docker, Docker Compose, GitHub Actions' },
]
</script>

<template>
  <div class="fl-container fl-section">
    <h1>Documentation</h1>
    <p class="docs__lede">
      FaultLab is a browser-based distributed systems simulator. These
      documents describe the product, the build plan, and the technical
      decisions behind it.
    </p>

    <h2 class="docs__heading">Documents</h2>
    <div class="fl-grid fl-grid--2">
      <div v-for="s in sections" :key="s.title" class="docs__card">
        <h3>{{ s.title }}</h3>
        <p>{{ s.desc }}</p>
      </div>
    </div>

    <h2 class="docs__heading">Technology stack</h2>
    <table class="docs__table">
      <thead>
        <tr>
          <th>Layer</th>
          <th>Technology</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="item in stack" :key="item.layer">
          <td>{{ item.layer }}</td>
          <td>{{ item.tech }}</td>
        </tr>
      </tbody>
    </table>

    <h2 class="docs__heading">Build progress</h2>
    <p class="docs__progress">
      The project follows a 30-day build plan. Each day delivers a concrete
      piece of working software. See the
      <a href="/docs/PLAN.md">build plan</a> for the full schedule.
    </p>
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
  background: var(--fl-bg-alt);
  border-left: 3px solid var(--fl-amber);
  padding: var(--fl-space-4);
}

.docs__card h3 {
  font-size: var(--fl-size-19);
  margin-bottom: var(--fl-space-2);
}

.docs__card p {
  font-size: var(--fl-size-16);
  color: var(--fl-grey-4);
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
