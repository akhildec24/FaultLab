/**
 * useCollab — WebSocket client composable for multiplayer collaboration.
 *
 * Connects to the Gleam collaboration server, handles:
 * - Room join/leave
 * - Document sync (graph state broadcast to peers)
 * - Presence (who's online, names, colors)
 * - Cursor sync (where peers are in the canvas)
 * - Reconnection with exponential backoff
 */

import { ref, onUnmounted, type Ref } from 'vue'
import type { GraphNode, GraphEdge } from '@/graph/types'

export interface Peer {
  id: string
  name: string
  color: string
  cursor: { x: number; y: number } | null
  lastSeen: number
}

export type CollabStatus = 'disconnected' | 'connecting' | 'connected' | 'reconnecting'

const PEER_COLORS = [
  '#f59e0b', '#10b981', '#3b82f6', '#ec4899',
  '#8b5cf6', '#ef4444', '#14b8a6', '#f97316',
]

const RECONNECT_BASE_DELAY = 1000
const RECONNECT_MAX_DELAY = 30000

export function useCollab(
  nodes: Ref<GraphNode[]>,
  edges: Ref<GraphEdge[]>,
) {
  const status = ref<CollabStatus>('disconnected')
  const roomId = ref<string | null>(null)
  const peers = ref<Map<string, Peer>>(new Map())
  const selfId = ref<string>('')
  const selfName = ref<string>('Anonymous')
  const serverUrl = ref<string>('ws://localhost:4000/ws')
  const lastError = ref<string>('')

  let ws: WebSocket | null = null
  let reconnectAttempts = 0
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null
  let presenceTimer: ReturnType<typeof setInterval> | null = null
  let cursorTimer: ReturnType<typeof setTimeout> | null = null
  let pendingCursor: { x: number; y: number } | null = null

  // --- Connection ---

  function connect(url?: string): void {
    if (url) serverUrl.value = url
    if (status.value === 'connected' || status.value === 'connecting') return

    status.value = reconnectAttempts > 0 ? 'reconnecting' : 'connecting'
    lastError.value = ''

    try {
      ws = new WebSocket(serverUrl.value)
    } catch (e) {
      lastError.value = `Failed to create WebSocket: ${e}`
      scheduleReconnect()
      return
    }

    ws.onopen = () => {
      status.value = 'connected'
      reconnectAttempts = 0
      selfId.value = `client-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
      // Auto-join room if set
      if (roomId.value) {
        send({ type: 'join', room: roomId.value })
      }
      startPresenceTimer()
    }

    ws.onmessage = (event) => {
      handleMessage(event.data)
    }

    ws.onerror = () => {
      lastError.value = 'WebSocket error'
    }

    ws.onclose = () => {
      status.value = 'disconnected'
      ws = null
      stopPresenceTimer()
      // Clear peers since we're disconnected
      peers.value = new Map()
      scheduleReconnect()
    }
  }

  function disconnect(): void {
    if (reconnectTimer) {
      clearTimeout(reconnectTimer)
      reconnectTimer = null
    }
    stopPresenceTimer()
    if (ws) {
      if (roomId.value) {
        send({ type: 'leave' })
      }
      ws.close()
      ws = null
    }
    status.value = 'disconnected'
    peers.value = new Map()
    reconnectAttempts = 0
  }

  function scheduleReconnect(): void {
    if (reconnectTimer) clearTimeout(reconnectTimer)
    const delay = Math.min(
      RECONNECT_BASE_DELAY * Math.pow(2, reconnectAttempts),
      RECONNECT_MAX_DELAY,
    )
    reconnectAttempts++
    reconnectTimer = setTimeout(() => connect(), delay)
  }

  // --- Room management ---

  function joinRoom(room: string): void {
    roomId.value = room
    if (status.value === 'connected') {
      send({ type: 'join', room })
      // Send current document state as sync response
      sendDocUpdate()
    }
  }

  function leaveRoom(): void {
    if (status.value === 'connected' && roomId.value) {
      send({ type: 'leave' })
    }
    roomId.value = null
    peers.value = new Map()
  }

  // --- Messaging ---

  function send(msg: unknown): void {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(msg))
    }
  }

  function handleMessage(raw: string): void {
    let msg: Record<string, unknown>
    try {
      msg = JSON.parse(raw)
    } catch {
      return
    }

    const type = msg.type as string
    const clientId = msg.client_id as string

    switch (type) {
      case 'peer_joined': {
        const peer: Peer = {
          id: clientId,
          name: `Peer ${clientId.slice(-4)}`,
          color: PEER_COLORS[peers.value.size % PEER_COLORS.length],
          cursor: null,
          lastSeen: Date.now(),
        }
        peers.value.set(clientId, peer)
        peers.value = new Map(peers.value)
        break
      }

      case 'peer_left': {
        peers.value.delete(clientId)
        peers.value = new Map(peers.value)
        break
      }

      case 'peer_presence': {
        const existing = peers.value.get(clientId)
        if (existing) {
          existing.name = (msg.data as string) || existing.name
          existing.lastSeen = Date.now()
          peers.value = new Map(peers.value)
        }
        break
      }

      case 'peer_cursor': {
        const existing = peers.value.get(clientId)
        if (existing) {
          try {
            const pos = JSON.parse(msg.data as string)
            existing.cursor = { x: pos.x, y: pos.y }
            existing.lastSeen = Date.now()
            peers.value = new Map(peers.value)
          } catch {
            // ignore malformed cursor
          }
        }
        break
      }

      case 'peer_update': {
        // A peer sent a document update — apply it
        try {
          const data = JSON.parse(msg.data as string)
          if (data.nodes && data.edges) {
            nodes.value = JSON.parse(JSON.stringify(data.nodes)) as GraphNode[]
            edges.value = JSON.parse(JSON.stringify(data.edges)) as GraphEdge[]
          }
        } catch {
          // ignore malformed update
        }
        break
      }

      case 'sync_request': {
        // A new peer is requesting the current document state
        sendDocUpdate()
        break
      }

      case 'sync_response': {
        // Initial document sync from an existing peer
        try {
          const data = JSON.parse(msg.data as string)
          if (data.nodes && data.edges) {
            nodes.value = JSON.parse(JSON.stringify(data.nodes)) as GraphNode[]
            edges.value = JSON.parse(JSON.stringify(data.edges)) as GraphEdge[]
          }
        } catch {
          // ignore
        }
        break
      }

      case 'error': {
        lastError.value = (msg.message as string) || 'Unknown server error'
        break
      }
    }
  }

  // --- Document sync ---

  function sendDocUpdate(): void {
    if (status.value !== 'connected' || !roomId.value) return
    const data = JSON.stringify({
      nodes: JSON.parse(JSON.stringify(nodes.value)),
      edges: JSON.parse(JSON.stringify(edges.value)),
    })
    send({ type: 'doc_update', data })
  }

  // --- Presence ---

  function setSelfName(name: string): void {
    selfName.value = name
    sendPresence()
  }

  function sendPresence(): void {
    if (status.value !== 'connected' || !roomId.value) return
    send({ type: 'presence', data: selfName.value })
  }

  function startPresenceTimer(): void {
    stopPresenceTimer()
    presenceTimer = setInterval(() => {
      sendPresence()
      // Prune stale peers (no presence in 10s)
      const now = Date.now()
      let changed = false
      for (const [id, peer] of peers.value) {
        if (now - peer.lastSeen > 10000) {
          peers.value.delete(id)
          changed = true
        }
      }
      if (changed) peers.value = new Map(peers.value)
    }, 5000)
  }

  function stopPresenceTimer(): void {
    if (presenceTimer) {
      clearInterval(presenceTimer)
      presenceTimer = null
    }
  }

  // --- Cursor sync (throttled) ---

  function sendCursor(x: number, y: number): void {
    pendingCursor = { x, y }
    if (cursorTimer) return
    cursorTimer = setTimeout(() => {
      if (pendingCursor && status.value === 'connected' && roomId.value) {
        send({ type: 'cursor', data: JSON.stringify(pendingCursor) })
      }
      pendingCursor = null
      cursorTimer = null
    }, 50)
  }

  // --- Peer list helpers ---

  function peerList(): Peer[] {
    return Array.from(peers.value.values())
  }

  // --- Cleanup ---

  onUnmounted(() => {
    disconnect()
  })

  return {
    // State
    status,
    roomId,
    peers,
    selfId,
    selfName,
    serverUrl,
    lastError,
    // Connection
    connect,
    disconnect,
    // Room
    joinRoom,
    leaveRoom,
    // Document sync
    sendDocUpdate,
    // Presence
    setSelfName,
    sendPresence,
    // Cursor
    sendCursor,
    // Helpers
    peerList,
  }
}
