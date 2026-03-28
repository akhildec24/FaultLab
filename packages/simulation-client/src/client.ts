/**
 * Typed client for the simulation Web Worker.
 *
 * Wraps `postMessage` / `onmessage` with a promise-based API.
 * Each request gets a unique `id` so responses can be matched.
 * Unsolicited `SimulationEvent` messages are forwarded to an
 * optional event handler.
 */

import type {
  WorkerRequest,
  WorkerResponse,
  WorkerMessage,
  WorkerPayload,
  SimulationEvent,
} from './protocol'

export type SimulationEventHandler = (event: SimulationEvent) => void
export type WorkerErrorHandler = (message: string) => void

export class SimulationWorkerClient {
  private worker: Worker
  private nextId = 1
  private pending = new Map<number, {
    resolve: (payload: WorkerPayload) => void
    reject: (error: Error) => void
  }>()
  private eventHandler: SimulationEventHandler | null = null
  private workerErrorHandler: WorkerErrorHandler | null = null

  constructor(worker: Worker) {
    this.worker = worker
    this.worker.onmessage = (e: MessageEvent<WorkerMessage>) => {
      this.handleMessage(e.data)
    }
    this.worker.onerror = (e) => {
      // Reject all pending requests on worker error
      for (const [, p] of this.pending) {
        p.reject(new Error(`Worker error: ${e.message}`))
      }
      this.pending.clear()
      if (this.workerErrorHandler) {
        this.workerErrorHandler(e.message)
      }
    }
  }

  /**
   * Set a handler for unsolicited simulation events
   * (e.g. `EVENTS`, `ENGINE_STOPPED`).
   */
  onEvent(handler: SimulationEventHandler | null): void {
    this.eventHandler = handler
  }

  /**
   * Set a handler for worker errors (crashes, uncaught exceptions).
   * The store uses this to trigger automatic worker recovery.
   */
  onWorkerError(handler: WorkerErrorHandler | null): void {
    this.workerErrorHandler = handler
  }

  private handleMessage(msg: WorkerMessage): void {
    // Check if it's a response (has `id`)
    if ('id' in msg) {
      const resp = msg as WorkerResponse
      const p = this.pending.get(resp.id)
      if (!p) return
      this.pending.delete(resp.id)
      if (resp.type === 'OK') {
        p.resolve(resp.payload)
      } else {
        p.reject(new Error(resp.message))
      }
    } else {
      // It's a SimulationEvent
      if (this.eventHandler) {
        this.eventHandler(msg as SimulationEvent)
      }
    }
  }

  private send(req: WorkerRequest): Promise<WorkerPayload> {
    const id = this.nextId++
    const fullReq = { ...req, id } as WorkerRequest
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject })
      this.worker.postMessage(fullReq)
    })
  }

  loadScenario(json: string): Promise<void> {
    return this.send({ type: 'LOAD_SCENARIO', json, id: 0 }).then(() => undefined)
  }

  start(): Promise<void> {
    return this.send({ type: 'START', id: 0 }).then(() => undefined)
  }

  pause(): Promise<void> {
    return this.send({ type: 'PAUSE', id: 0 }).then(() => undefined)
  }

  reset(): Promise<void> {
    return this.send({ type: 'RESET', id: 0 }).then(() => undefined)
  }

  step(): Promise<boolean> {
    return this.send({ type: 'STEP', id: 0 }).then((p) => {
      if (p.kind === 'boolean') return p.value
      throw new Error('Unexpected response kind for STEP')
    })
  }

  run(maxSteps: number): Promise<number> {
    return this.send({ type: 'RUN', maxSteps, id: 0 }).then((p) => {
      if (p.kind === 'number') return p.value
      throw new Error('Unexpected response kind for RUN')
    })
  }

  getMetrics(): Promise<string> {
    return this.send({ type: 'GET_METRICS', id: 0 }).then((p) => {
      if (p.kind === 'metrics') return p.json
      throw new Error('Unexpected response kind for GET_METRICS')
    })
  }

  getState(): Promise<string> {
    return this.send({ type: 'GET_STATE', id: 0 }).then((p) => {
      if (p.kind === 'state') return p.json
      throw new Error('Unexpected response kind for GET_STATE')
    })
  }

  getRecentEvents(): Promise<string> {
    return this.send({ type: 'GET_RECENT_EVENTS', id: 0 }).then((p) => {
      if (p.kind === 'events') return p.json
      throw new Error('Unexpected response kind for GET_RECENT_EVENTS')
    })
  }

  getStatus(): Promise<{ running: boolean; currentTime: number; pendingEvents: number }> {
    return this.send({ type: 'GET_STATUS', id: 0 }).then((p) => {
      if (p.kind === 'status') return { running: p.running, currentTime: p.currentTime, pendingEvents: p.pendingEvents }
      throw new Error('Unexpected response kind for GET_STATUS')
    })
  }

  injectFailure(json: string): Promise<void> {
    return this.send({ type: 'INJECT_FAILURE', json, id: 0 }).then(() => undefined)
  }

  terminate(): void {
    this.worker.terminate()
    for (const [, p] of this.pending) {
      p.reject(new Error('Worker terminated'))
    }
    this.pending.clear()
  }
}
