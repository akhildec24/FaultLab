# FaultLab

A browser-based distributed systems simulator. Design a software
architecture, send simulated traffic through it, inject failures, and
watch the consequences unfold — all in the browser, no backend required
for simulation.

## Features

- **Visual graph editor** — drag-and-drop topology builder with pan/zoom,
  node types (client, service, database), configurable per-node and
  per-edge properties (capacity, latency, error rate, queue limits,
  retry policies, shedding, replication).
- **Rust → WASM simulation engine** — discrete-event engine running in
  a Web Worker, off the main thread. Handles request generation, queueing,
  timeouts, retries, shedding, failures, and replication lag.
- **Scenario language** — text-based topology definition with syntax
  highlighting, error underlining, and visual-to-code sync.
- **Real-time dashboards** — throughput, error rate, latency percentiles,
  queue depth, utilisation. Compare two simulation runs side by side.
- **Event timeline** — searchable, filterable event log with timestamps.
- **Deterministic replay** — scenario version, random seed, command log,
  and failure log for exact incident reproduction.
- **Local-first storage** — IndexedDB persistence, auto-save, history
  snapshots, import/export JSON.
- **Multiplayer collaboration** — Gleam/Erlang WebSocket server with
  room-based presence, cursor sync, and document synchronisation.
- **Performance** — viewport culling for 100+ node scenarios, worker
  recovery with exponential backoff, throttled cursor sync.

## Quick start

### Prerequisites

- Node.js 22+
- Rust (stable) with `wasm32-unknown-unknown` target
- Gleam 1.17+ and Erlang/OTP (for collaboration server)

### Development

```sh
# Install web dependencies
cd apps/web
npm install

# Build the WASM engine (from project root)
cargo build --release -p simulation-wasm

# Start the dev server
npm run dev
```

Open http://localhost:5173

### Collaboration server (optional)

```sh
cd services/collaboration-server
gleam deps
gleam run
```

WebSocket server on port 4000, health check at `GET /health`.

### Docker

```sh
docker compose up --build
```

- Web app: http://localhost:5173
- Collaboration server: ws://localhost:4000/ws
- Health check: http://localhost:4000/health

## Architecture

```
┌─────────────────────────────────────────────┐
│  Vue 3 + TypeScript + Pinia (UI)            │
│  ├── Graph editor (SVG canvas)              │
│  ├── Code editor (scenario language)        │
│  ├── Dashboards & event timeline            │
│  └── Presence bar & peer cursors            │
├─────────────────────────────────────────────┤
│  Web Worker (off-main-thread)               │
│  └── Rust → WASM simulation engine          │
│      (discrete-event, deterministic)        │
├─────────────────────────────────────────────┤
│  IndexedDB (local-first storage)            │
│  Automerge CRDT (document sync)             │
├─────────────────────────────────────────────┤
│  WebSocket                                  │
│  └── Gleam + Erlang/OTP (collab server)     │
│      (rooms, presence, document relay)      │
└─────────────────────────────────────────────┘
```

### Project structure

```
apps/web/           Vue 3 frontend (Vite, Pinia, TypeScript)
crates/             Rust workspace
  simulation/       Core simulation engine (discrete-event)
  simulation-wasm/  WASM bindings (wasm-bindgen)
  simulation-fuzz/  Fuzz targets
packages/
  simulation-client/  TS worker client (promise-based API)
services/
  collaboration-server/  Gleam WebSocket server (mist, OTP actors)
infrastructure/
  docker/           Dockerfiles (web + server)
  deployment/       docker-compose.yml
docs/
  SPEC.md           Product specification
  PLAN.md           30-day build plan
  PROGRESS.md       Build progress log
```

## Testing

```sh
# Rust tests (146 tests)
cargo test --workspace

# Gleam tests (12 tests)
cd services/collaboration-server && gleam test

# Web type checking
cd apps/web && npx vue-tsc --noEmit
```

## CI

GitHub Actions runs on every push to `main`:
- **Rust**: format check, clippy, `cargo test --workspace`
- **Gleam**: `gleam build`, `gleam test`
- **Web**: `npm ci`, `vue-tsc --noEmit`, `npm run build`

## Documentation

- [Product specification](docs/SPEC.md)
- [30-day build plan](docs/PLAN.md)
- [Build progress](docs/PROGRESS.md)

## Tech stack

| Layer | Technology |
|-------|-----------|
| Frontend | Vue 3, TypeScript, Vite, Pinia |
| Simulation | Rust → WebAssembly, Web Worker |
| Collaboration | Gleam, Erlang/OTP, mist WebSocket |
| Storage | IndexedDB, Automerge CRDT |
| Deployment | Docker, docker-compose |

