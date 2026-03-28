# FaultLab — Build Progress

Tracking day-by-day progress against the 30-day plan in `docs/PLAN.md`.

## Week 1 — Architecture and simulation foundations

- [x] **Day 1** — Define the product. Spec written (`docs/SPEC.md`), six
      component types chosen, first incident scenario defined. Vue + Vite +
      TypeScript app scaffolded with GOV.UK styling.
- [x] **Day 2** — Create the monorepo. Rust workspace (3 crates:
      simulation-core, simulation-wasm, scenario-parser) with 10 passing
      tests. Gleam collaboration server scaffolded. Shared TypeScript
      packages (protocol, scenario-schema, example-scenarios). Justfile
      with `just dev`. Docker Compose. GitHub Actions CI. Formatting
      (rustfmt, clippy, prettier, gleam format).
- [x] **Day 3** — Design the simulation model. Split types into
      immutable config (NodeConfig, ConnectionConfig, TrafficConfig,
      RetryPolicy) and mutable runtime (RequestState, NodeRuntimeState,
      SimulationState). Added request lifecycle phases, failure injection
      types, retry strategies. 19 tests passing. ADR-001 written.
- [x] **Day 4** — Build the event scheduler. Deterministic RNG
      (splitmix64), traffic generator (linear ramp, per-second event
      scheduling), request routing (connection graph, next-hop, packet
      loss, jitter), timeout scheduling, retry logic (immediate/fixed/
      exponential with jitter and budget). Engine now drives full
      simulations. 52 tests passing, clippy clean.
- [x] **Day 5** — Simulate a basic request. Fixed multi-hop routing
      (client → service → database). Requests now flow through the full
      chain: intermediate nodes forward downstream, leaf nodes complete.
      Per-hop latency recorded. Packet loss in transit handled at every
      stage. Retry storm scenario runs to completion with correct metric
      accounting. 59 tests passing, clippy clean.
- [x] **Day 6** — Add the network model. New `network.rs` module with
      `ConnectionState` and `NetworkState`. Token-bucket bandwidth
      limiting per connection. Connection up/down state for network
      partitions. `ConnectionFailed`/`ConnectionRestored` events.
      Injected latency and packet loss (mid-simulation failure
      injection). Engine routes all transit through the network model.
      `Scheduler::peek` for non-destructive inspection. 80 tests
      passing, clippy clean.
- [x] **Day 7** — Test and document the engine. Property-based tests
      with proptest (deterministic replay, metric invariants, latency
      bounds, RNG properties). Edge-case tests (empty scenario, missing
      nodes, zero capacity, diamond topology, high packet loss, pause/
      reset). ADR-002 documents the full event flow. Engine fixes:
      `start_processing` handles nonexistent nodes, `run()` respects
      paused state. 98 tests passing, clippy clean.

## Week 2 — WebAssembly and the visual editor

- [x] **Day 8** — Compile Rust to WebAssembly. Full WASM API:
      `loadScenario`, `start`, `step`, `run`, `pause`, `reset`,
      `isRunning`, `currentTime`, `getMetrics`, `getState`,
      `getRecentEvents`, `pendingEvents`. Engine gains recent-events
      ring buffer (256 cap) and `pending()` accessor. Panic hook via
      `console_error_panic_hook`. TypeScript wrapper package
      (`@faultlab/simulation-client`) with typed interfaces. 11
      wasm-bindgen-test API surface tests. `wasm32-unknown-unknown`
      target builds. `just wasm-pack` target added. 98 native tests
      passing, clippy clean.
- [x] **Day 9** — Move engine into a Web Worker. Typed message
      protocol with request IDs for matching responses
      (`LOAD_SCENARIO`, `START`, `PAUSE`, `RESET`, `STEP`, `RUN`,
      `GET_METRICS`, `GET_STATE`, `GET_RECENT_EVENTS`, `GET_STATUS`).
      Worker drains recent events after each step/run and sends
      unsolicited `EVENTS` messages. Promise-based
      `SimulationWorkerClient` with typed methods. Pinia store
      (`stores/simulation.ts`) with reactive state mirroring the
      worker. Vite config for ES module workers and WASM bundling.
      TypeScript path aliases for `@faultlab/simulation-client`.
- [x] **Day 10** — Build the graph editor. SVG canvas with pan/zoom
      (wheel zoom toward cursor, drag-to-pan). Add nodes (client,
      service, database) with colour-coded shapes and icons. Drag
      to reposition, click to select, Delete to remove. Connect
      nodes via amber handle on hover (bezier curve edges). Pinia
      graph store with full CRUD. Toolbar with zoom controls and
      node/edge count. Grid background. Keyboard shortcuts
      (Delete, Escape).
- [x] **Day 11** — Build the configuration inspector. NodeInspector
      edits name, kind, capacity, latency, error rate, timeout, queue
      limit with inline validation and hint text. EdgeInspector edits
      latency, packet loss, bandwidth with route display. Split layout
      in EditorView (canvas + 320px inspector panel). GraphNode and
      GraphEdge extended with config properties mirroring Rust
      NodeConfig/ConnectionConfig. Default configs per node kind.
      Graph store gains updateNode/updateEdge actions.
- [x] **Day 12** — Connect graph to Rust. Graph-to-scenario converter
      (`converter.ts`) transforms visual nodes/edges into Scenario JSON
      matching Rust `NodeConfig`/`ConnectionConfig`/`Scenario` types.
      Validation: empty graph, duplicate names, invalid capacities,
      error rates, timeouts, missing node references, isolated nodes
      (warnings). `SimulationControls.vue` with Run/Pause/Step/Run 500/
      Reset buttons, live status (running, time, pending), and metrics
      grid (requests, success, failed, timeouts, dropped, avg/p95
      latency). Wired into EditorView between header and canvas.
- [x] **Day 13** — Animate request movement. Animation store
      (`stores/animation.ts`) processes sim events into request
      particles (interpolated along edges via requestAnimationFrame)
      and node flash effects (colour-coded by event type, fading over
      800ms). Particle colours: transit=amber, processing=indigo,
      success=green, failed/timeout=red, queued=grey. Speed control
      (0.5x/1x/2x/4x) in SimulationControls. SVG overlay layer in
      GraphEditor renders particles and flashes on top of nodes/edges.
      Simulation store gains `onEvents` callback hook. EditorView
      wires events → animation store on mount.
- [x] **Day 14** — First complete demonstration. Three preset scenarios
      (Overloaded Database, Cascading Failure, Network Partition) with
      full topologies and tuned config. PresetSelector dropdown in
      SimulationControls — loads preset, clears sim state. Simulation
      clock display (ms/s formatting, amber). Clear button resets graph
      and sim. End-to-end flow: select preset → Run → watch particles,
      flashes, metrics, clock update in real time.

## Week 3 — Failures, reliability patterns, observability

- [x] **Day 15** — Failure injection. Engine `inject_failure` method
      processes `FailureInjection` enum (crash, recover, add latency,
      add packet loss, disconnect, reduce capacity) and records events
      (NodeFailed, NodeRecovered, ConnectionFailed) in event history.
      WASM binding `injectFailure(json)`, worker protocol `INJECT_FAILURE`
      message, client `injectFailure()` method, simulation store action.
      FailurePanel.vue component with dropdown for failure type, node/edge
      selectors, parameter inputs (latency ms, packet loss rate, capacity).
      Integrated into SimulationControls, shown when sim is loaded.
      6 new Rust tests (107 total).
- [x] **Day 16** — Retries and timeouts. RetryPolicy exposed in graph
      types (RetryStrategyType, RetryPolicy with strategy/max_retries/
      jitter/budget). Converter passes retry_policy to scenario JSON
      with budget field. NodeInspector gains Retry Policy section
      (strategy dropdown, max retries, jitter, budget input). Default
      policies per kind: client=immediate/3, service=exponential/3/0.2,
      database=fixed/1. New "Retry Storm" preset (aggressive client,
      30% error rate, immediate retries, no budget). 4 new Rust tests
      (retry storm, budget exhaustion, exponential backoff delays,
      zero max_retries). 111 tests total.
- [x] **Day 17** — Queues and load shedding. SheddingPolicy enum
      (Drop, Reject, Backpressure) added to NodeConfig with
      #[serde(tag="type")] tagged representation. Engine RequestArrived
      handler enqueues requests when at capacity and queue has space,
      applies shedding policy when queue is full. RequestShedded event
      handler: Drop silently drops, Reject triggers retry, Backpressure
      drops without retry. RequestDequeued event starts processing.
      try_dequeue method called from finish_processing and timeout
      handler to dequeue waiting requests when capacity frees.
      waiting_queues (HashMap<String, VecDeque<u64>>) in SimulationState.
      shedded counter in Metrics and NodeRuntimeState. TS graph types
      gain SheddingPolicyType and shed_policy field. Converter passes
      shed_policy as {type: "drop"|"reject"|"backpressure"}. NodeInspector
      gains Load Shedding section with policy dropdown. New "Queue
      Overflow" preset (burst client, capacity=5 service with queue=10,
      reject policy, cache store). 4 new Rust tests (queue dequeue,
      drop/reject/backpressure policies). 115 tests total.
- [x] **Day 18** — Caches and replication. ReplicationRole enum
      (Standalone, Leader, Replica) added to NodeConfig with
      replication_lag_ms field. Cache hit/miss logic in
      start_processing: nodes with cache_hit_rate roll RNG on each
      request — hits complete immediately (1ms) without downstream
      forwarding, misses proceed normally. CacheHit, CacheMiss, and
      StaleRead events added. Replica nodes with replication_lag_ms > 0
      have 30% chance of serving stale reads. Metrics track cache_hits,
      cache_misses, stale_reads. NodeRuntimeState tracks per-node
      total_cache_hits, total_cache_misses, total_stale_reads. TS graph
      types gain ReplicationRoleType and replication fields. Converter
      passes replication_role and replication_lag_ms to scenario JSON.
      NodeInspector gains Replication section (role dropdown, lag input).
      New "Cache & Replication" preset (cache layer, leader DB, replica
      with 300ms lag). 5 new Rust tests (cache hit/miss, 100% hit rate,
      0% hit rate, stale reads with lag, no stale reads without lag).
      120 tests total.
- [x] **Day 19** — Deterministic replay. CommandLogEntry enum
      (Start, Step, Run, Pause, Reset, InjectFailure) with virtual
      time_ms timestamps. CommandLog struct records all actions during
      a simulation run. ReplayMetadata captures scenario_version,
      engine_version, seed, total_steps, final_time_ms. ReplayRecording
      bundles scenario + command log + metadata for serialisation.
      ENGINE_VERSION and SCENARIO_VERSION constants for compatibility
      checking. Engine records actions in start(), step(), run(), pause(),
      reset(), inject_failure(). Engine::recording() builds a complete
      recording. Engine::replay() replays a recording by re-executing
      each command log entry on a fresh engine — produces identical
      metrics. 5 new Rust tests (replay identical metrics, replay with
      failure injection, recording metadata correctness, command log
      records actions, recording JSON serialisation round-trip).
      125 tests total.
- [x] **Day 20** — Event timeline. EventTimeline.vue component with
      searchable, filterable event log. Accumulates events in
      simulation store (eventLog with 5000 cap). Filters: text search
      (matches type, node ID), event type dropdown (auto-populated
      from actual events), component/node filter (auto-populated),
      request ID filter. Colour-coded badges per event type (green
      for success, red for failure, amber for retry, blue for
      request lifecycle, grey for queue). Collapsible bottom panel
      in EditorView with toggle button showing live event count.
      Most recent events shown first, 200 entry display limit.
      Clear filters button. 125 tests total.
- [x] **Day 21** — Dashboards and comparison mode. Dashboard.vue
      component with real-time KPI cards (throughput, success rate,
      error rate, avg/p50/p95/p99 latency, cache hit rate). Request
      counts grid (total, success, failed, timed out, dropped,
      shedded, retries, stale reads). Queue depth bar chart with
      per-node bars. Node utilisation bar chart with colour-coded
      thresholds (green <50%, amber 50-80%, red >80%). Snapshot
      system — take snapshots of current metrics, compare two
      snapshots side-by-side with delta values (requests, success,
      failures, latency, throughput). Bottom panel in EditorView
      with tabbed interface (Timeline / Dashboard). TS Metrics
      interface updated with shedded, cache_hits, cache_misses,
      stale_reads fields. 125 tests total.

## Week 4 — Language, collaboration, polish

- [x] **Day 22** — Design the scenario language. DSL grammar spec
      in docs/DSL.md covering: scenario declaration with name and
      seed, nodes section (6 component kinds: client, service, queue,
      cache, database, external_api), node properties (capacity,
      latency, error_rate, timeout, queue_limit, cache_hit_rate,
      replication role/lag, retry policy, shed policy), edges section
      with latency/packet_loss/bandwidth, traffic section with
      start/target/ramp, failures section with scheduled crash/recover/
      add_latency/disconnect/add_packet_loss/reduce_capacity. Duration
      syntax (ms/s/m), percent syntax (0%-100%), fraction syntax for
      jitter. Retry policies: immediate, fixed, exponential with
      max_retries/jitter/budget. Shed policies: drop, reject,
      backpressure. Replication roles: standalone, leader, replica.
      EBNF grammar. Two example .fault files (retry-storm,
      cache-replication). Error reporting spec with line/column
      numbers and suggestions.
- [x] **Day 23** — Build the parser. Full lexer, parser, AST, and
      semantic validation for the FaultLab DSL. Lexer tokenizes
      keywords, strings, integers, floats, durations (ms/s/m),
      percents, arrows, braces, comments (# and //). Recursive
      descent parser produces AST with line/column error tracking.
      AST types: AstScenario, AstNode, AstEdge, AstTraffic,
      AstFailure, AstRetryPolicy. AST→Scenario conversion via
      to_scenario(). Semantic validation: duplicate node IDs, unknown
      edge references, unknown failure references. Public API:
      parse_dsl(), parse_dsl_with_failures(), parse_dsl_ast().
      24 parser tests (lexer, parser, validation, round-trip).
      146 tests total, clippy clean.
- [x] **Day 24** — Build the code editor. CodeEditor.vue with
      syntax highlighting (keywords, strings, numbers, durations,
      percents, comments, braces, arrows), line numbers with error
      markers, error underlining (wavy red), tab-to-spaces, copy
      button. Visual-to-code sync: graphToDsl() generates .fault
      source from visual graph, auto-updates when graph changes.
      Sync mode indicator (synced vs edited). Re-sync button.
      validateDsl() basic TS-side validation (unclosed strings,
      missing scenario, unbalanced braces). Third tab in bottom
      panel (Timeline / Dashboard / Code). 146 tests total.
- [x] **Day 25** — Local-first storage. IndexedDB wrapper
      (apps/web/src/storage/db.ts) with two object stores: scenarios
      (CRUD) and history (snapshots keyed by scenarioId+timestamp).
      useStorage composable with debounced auto-save (2s), periodic
      history snapshots (30s), manual snapshots, restore from
      history, scenario list/load/delete, import/export JSON
      (download + file upload), online/offline indicator. StoragePanel
      component with status bar, action buttons (New, Save, Export,
      Import, Snapshot), collapsible saved scenarios list, collapsible
      history list with restore. Integrated into EditorView as
      toggleable sidebar panel via Storage button in toolbar.
      146 tests total.
- [x] **Day 26** — Gleam collaboration server. WebSocket server
      using mist v6 on port 4000 with /ws (WebSocket upgrade) and
      /health (health check) endpoints. rooms.gleam module: RoomState
      with client dict and document state, RoomMessage (Join, Leave,
      Update, Cursor, Presence), OutMessage (PeerUpdate, PeerCursor,
      PeerPresence, PeerJoined, PeerLeft, SyncRequest, SyncResponse).
      Pure handle_room_message function for testability. JSON
      encoding/decoding via gleam_json v3 with gleam/dynamic/decode.
      Actor-based room management with named subjects for room lookup.
      WebSocket handler: client ID generation, inbox subject for
      outgoing messages, selector for receiving actor messages,
      on_close cleanup. Document sync: new clients receive current
      document, existing clients get sync requests. 12 Gleam tests
      (room creation, message parsing for all types, error handling,
      JSON encoding). 146 Rust tests total.
- [x] **Day 27** — Multiplayer presence. useCollab composable
      (apps/web/src/collab/useCollab.ts) — WebSocket client with
      room join/leave, document sync (broadcast graph state),
      presence (name, color, last-seen pruning), cursor sync
      (throttled 50ms), reconnection with exponential backoff
      (1s→30s max). PresenceBar component showing connection status
      dot and peer avatars with names. PeerCursors overlay showing
      remote cursors with colored labels. Integrated into EditorView
      with Collaborate toggle button, debounced graph change
      broadcast (500ms), and peer cursor overlay on canvas.
- [x] **Day 28** — Performance and resilience. Large scenario
      generator (generateLargeScenario) producing layered 120-node
      topologies (clients → services → databases) with correct
      GraphNode/GraphEdge types. Viewport culling in GraphEditor:
      only renders nodes/edges within the visible viewport when
      node count exceeds 50, with 100px padding. Worker recovery:
      SimulationWorkerClient.onWorkerError callback, simulation
      store tracks lastLoadedScenario and workerHealthy, automatic
      worker restart with exponential backoff (1s→10s max),
      scenario reload on recovery. Toolbar "120 nodes" button and
      worker recovery warning indicator. 146 Rust tests, 12 Gleam
      tests.
- [x] **Day 29** — Testing, deployment, documentation. Technical
      README with features, architecture diagram, project structure,
      testing commands, CI description, tech stack table. Docker:
      server Dockerfile updated with gleam build step, correct port
      4000, HEALTHCHECK on /health endpoint; web Dockerfile updated
      to multi-stage build (build → preview) with health check;
      docker-compose updated with correct ports, healthcheck config,
      depends_on with service_healthy condition. CI workflow already
      covers Rust (fmt, clippy, test), Gleam (build, test), Web
      (vue-tsc, build). 146 Rust tests, 12 Gleam tests, web type
      checks clean.
- [ ] **Day 30** — Portfolio presentation.
