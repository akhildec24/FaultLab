/**
 * Simulation Web Worker (local entry for Vite ?worker resolution).
 *
 * Inlined from packages/simulation-client/src/worker.ts because
 * Vite ?worker imports can't resolve package aliases.
 */

import type { WorkerRequest, WorkerResponse, WorkerMessage, WorkerPayload } from '@faultlab/simulation-client/protocol'

type WasmModule = {
  Simulation: new () => {
    loadScenario(json: string): void
    start(): void
    pause(): void
    reset(): void
    step(): boolean
    run(maxSteps: number): number
    isRunning(): boolean
    currentTime(): number
    getMetrics(): string
    getState(): string
    getRecentEvents(): string
    pendingEvents(): number
    injectFailure(json: string): void
  }
}

let sim: InstanceType<WasmModule['Simulation']> | null = null
let initialized = false

async function ensureLoaded(): Promise<void> {
  if (initialized) return
  // The WASM stub at src/wasm/simulation_wasm.js is replaced by
  // `just wasm-pack`. Use relative path for worker context.
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const mod = (await import('./wasm/simulation_wasm.js')) as any as WasmModule
  sim = new mod.Simulation()
  initialized = true
}

function ok(id: number, payload: WorkerPayload): WorkerResponse {
  return { id, type: 'OK', payload }
}

function err(id: number, message: string): WorkerResponse {
  return { id, type: 'ERROR', message }
}

async function handleRequest(req: WorkerRequest): Promise<WorkerResponse> {
  await ensureLoaded()
  if (!sim) return err(req.id, 'Engine not initialised')

  try {
    switch (req.type) {
      case 'LOAD_SCENARIO':
        sim.loadScenario(req.json)
        return ok(req.id, { kind: 'void' })

      case 'START':
        sim.start()
        return ok(req.id, { kind: 'void' })

      case 'PAUSE':
        sim.pause()
        return ok(req.id, { kind: 'void' })

      case 'RESET':
        sim.reset()
        return ok(req.id, { kind: 'void' })

      case 'STEP': {
        const processed = sim.step()
        drainEvents()
        return ok(req.id, { kind: 'boolean', value: processed })
      }

      case 'RUN': {
        const steps = sim.run(req.maxSteps)
        drainEvents()
        return ok(req.id, { kind: 'number', value: steps })
      }

      case 'GET_METRICS':
        return ok(req.id, { kind: 'metrics', json: sim.getMetrics() })

      case 'GET_STATE':
        return ok(req.id, { kind: 'state', json: sim.getState() })

      case 'GET_RECENT_EVENTS':
        return ok(req.id, { kind: 'events', json: sim.getRecentEvents() })

      case 'GET_STATUS':
        return ok(req.id, {
          kind: 'status',
          running: sim.isRunning(),
          currentTime: sim.currentTime(),
          pendingEvents: sim.pendingEvents(),
        })

      case 'INJECT_FAILURE':
        sim.injectFailure(req.json)
        drainEvents()
        return ok(req.id, { kind: 'void' })

      default: {
        const r = req as { id: number; type: string }
        return err(r.id, `Unknown request type: ${r.type}`)
      }
    }
  } catch (e) {
    return err(req.id, String(e))
  }
}

function drainEvents(): void {
  if (!sim) return
  const json = sim.getRecentEvents()
  try {
    const events = JSON.parse(json) as unknown[]
    if (events.length > 0) {
      const msg: WorkerMessage = { type: 'EVENTS', events }
      postMessage(msg)
    }
  } catch {
    // Ignore parse errors
  }
}

self.onmessage = async (e: MessageEvent<WorkerRequest>) => {
  const response = await handleRequest(e.data)
  postMessage(response)
}
