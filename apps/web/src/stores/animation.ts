/**
 * Pinia store for request animation.
 *
 * Consumes simulation events from the simulation store and turns
 * them into animated particles that travel along graph edges.
 * Uses requestAnimationFrame for smooth interpolation.
 */

import { defineStore } from 'pinia'
import { ref, shallowRef } from 'vue'
import type { RequestParticle, NodeFlash, SimEvent, SpeedMultiplier } from '@/graph/animation'
import { PARTICLE_COLORS, FLASH_DURATION } from '@/graph/animation'

export const useAnimationStore = defineStore('animation', () => {
  // --- State ---
  const particles = shallowRef<RequestParticle[]>([])
  const flashes = shallowRef<NodeFlash[]>([])
  const speed = ref<SpeedMultiplier>(1)
  const lastFrameTime = ref(0)

  // --- Animation loop ---
  let rafId: number | null = null
  let running = false

  /** Map of requestId → current node (for tracking position). */
  const requestPositions = new Map<number, string>()

  function startLoop() {
    if (running) return
    running = true
    lastFrameTime.value = performance.now()
    rafId = requestAnimationFrame(tick)
  }

  function stopLoop() {
    running = false
    if (rafId !== null) {
      cancelAnimationFrame(rafId)
      rafId = null
    }
  }

  function tick(now: number) {
    if (!running) return
    const dt = (now - lastFrameTime.value) / 1000 // seconds
    lastFrameTime.value = now

    // Advance particles
    const speedPerSec = 0.8 * speed.value // 0.8 progress per second at 1x
    const updated = particles.value.map((p) => ({
      ...p,
      progress: p.progress + dt * speedPerSec,
    })).filter((p) => p.progress < 1)

    particles.value = updated

    // Advance flashes
    const dtMs = dt * 1000
    flashes.value = flashes.value
      .map((f) => ({ ...f, remaining: f.remaining - dtMs }))
      .filter((f) => f.remaining > 0)

    rafId = requestAnimationFrame(tick)
  }

  // --- Process events from simulation ---
  function processEvents(events: unknown[]): void {
    if (events.length === 0) return

    const newParticles: RequestParticle[] = []
    const newFlashes: NodeFlash[] = []

    for (const raw of events) {
      const evt = raw as SimEvent
      if (!evt || typeof evt.time !== 'number' || !evt.event) continue
      const e = evt.event
      const type = e.type

      switch (type) {
        case 'request_created': {
          // Request starts at origin node
          if (e.origin && e.request_id !== undefined) {
            requestPositions.set(e.request_id, e.origin)
          }
          break
        }
        case 'request_in_transit': {
          // This event isn't in the public enum but handle it just in case
          if (e.from && e.to && e.request_id !== undefined) {
            newParticles.push({
              requestId: e.request_id,
              fromId: e.from,
              toId: e.to,
              progress: 0,
              status: 'transit',
            })
            requestPositions.set(e.request_id, e.to)
          }
          break
        }
        case 'request_arrived': {
          if (e.node_id) {
            newFlashes.push({
              nodeId: e.node_id,
              color: PARTICLE_COLORS.transit,
              remaining: FLASH_DURATION,
            })
          }
          break
        }
        case 'request_started': {
          if (e.node_id) {
            newFlashes.push({
              nodeId: e.node_id,
              color: PARTICLE_COLORS.processing,
              remaining: FLASH_DURATION,
            })
          }
          break
        }
        case 'request_completed': {
          if (e.node_id) {
            newFlashes.push({
              nodeId: e.node_id,
              color: e.success ? PARTICLE_COLORS.success : PARTICLE_COLORS.failed,
              remaining: FLASH_DURATION,
            })
          }
          if (e.request_id !== undefined) {
            requestPositions.delete(e.request_id)
          }
          break
        }
        case 'request_timed_out': {
          if (e.node_id) {
            newFlashes.push({
              nodeId: e.node_id,
              color: PARTICLE_COLORS.timeout,
              remaining: FLASH_DURATION,
            })
          }
          if (e.request_id !== undefined) {
            requestPositions.delete(e.request_id)
          }
          break
        }
        case 'message_queued': {
          if (e.queue_id) {
            newFlashes.push({
              nodeId: e.queue_id,
              color: PARTICLE_COLORS.queued,
              remaining: FLASH_DURATION,
            })
          }
          break
        }
        case 'message_dropped': {
          if (e.queue_id) {
            newFlashes.push({
              nodeId: e.queue_id,
              color: PARTICLE_COLORS.failed,
              remaining: FLASH_DURATION * 0.5,
            })
          }
          break
        }
        case 'node_failed': {
          if (e.node_id) {
            newFlashes.push({
              nodeId: e.node_id,
              color: PARTICLE_COLORS.failed,
              remaining: FLASH_DURATION * 2,
            })
          }
          break
        }
        case 'node_recovered': {
          if (e.node_id) {
            newFlashes.push({
              nodeId: e.node_id,
              color: PARTICLE_COLORS.success,
              remaining: FLASH_DURATION * 2,
            })
          }
          break
        }
        default:
          // Unknown event type — ignore
          break
      }
    }

    // Merge new particles and flashes with existing
    if (newParticles.length > 0) {
      particles.value = [...particles.value, ...newParticles]
    }
    if (newFlashes.length > 0) {
      flashes.value = [...flashes.value, ...newFlashes]
    }

    // Start animation loop if we have particles or flashes
    if ((particles.value.length > 0 || flashes.value.length > 0) && !running) {
      startLoop()
    }
  }

  function setSpeed(s: SpeedMultiplier): void {
    speed.value = s
  }

  function clear(): void {
    particles.value = []
    flashes.value = []
    requestPositions.clear()
    stopLoop()
  }

  return {
    // State
    particles,
    flashes,
    speed,
    // Actions
    processEvents,
    setSpeed,
    clear,
    startLoop,
    stopLoop,
  }
})
