/**
 * Simulation Web Worker (local entry for Vite ?worker resolution).
 *
 * Loads the WASM simulation engine and processes typed messages.
 */

import type { WorkerRequest, WorkerResponse, WorkerMessage, WorkerPayload } from '@faultlab/simulation-client/protocol'
import initWasm, { Simulation } from './wasm/simulation_wasm.js'

let sim: Simulation | null = null
let initialized = false

async function ensureLoaded(): Promise<void> {
  if (initialized) return
  await initWasm()
  sim = new Simulation()
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
        sim.load_scenario(req.json)
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
        return ok(req.id, { kind: 'metrics', json: sim.get_metrics() })

      case 'GET_STATE':
        return ok(req.id, { kind: 'state', json: sim.get_state() })

      case 'GET_RECENT_EVENTS':
        return ok(req.id, { kind: 'events', json: sim.get_recent_events() })

      case 'GET_STATUS':
        return ok(req.id, {
          kind: 'status',
          running: sim.is_running(),
          currentTime: Number(sim.current_time()),
          pendingEvents: sim.pending_events(),
        })

      case 'INJECT_FAILURE':
        sim.inject_failure(req.json)
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
  const json = sim.get_recent_events()
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
