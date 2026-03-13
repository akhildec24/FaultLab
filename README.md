# FaultLab

A browser-based distributed systems simulator. Design a software
architecture, send simulated traffic through it, inject failures, and
watch the consequences unfold.

## Quick start

```sh
cd apps/web
npm install
npm run dev
```

Open http://localhost:5173

## Documentation

- [Product specification](docs/SPEC.md)
- [30-day build plan](docs/PLAN.md)
- [Build progress](docs/PROGRESS.md)

## Architecture

```
Vue 3 + TypeScript (interface)
        ↓
Web Worker (off-main-thread)
        ↓
Rust → WebAssembly (simulation engine)

Automerge CRDT (local-first documents)
        ↓
WebSocket
        ↓
Gleam + Erlang/OTP (collaboration server)
```

