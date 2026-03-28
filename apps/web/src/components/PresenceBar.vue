<template>
  <div class="presence-bar" v-if="peers.size > 0 || status !== 'disconnected'">
    <div class="presence-status">
      <span
        class="presence-dot"
        :class="`presence-dot--${status}`"
        :title="statusLabel"
      />
      <span class="presence-label">{{ statusLabel }}</span>
    </div>
    <div class="presence-peers" v-if="peerList.length > 0">
      <div
        v-for="peer in peerList"
        :key="peer.id"
        class="presence-peer"
        :title="peer.name"
      >
        <span class="presence-avatar" :style="{ background: peer.color }">
          {{ peer.name.charAt(0).toUpperCase() }}
        </span>
        <span class="presence-name">{{ peer.name }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { CollabStatus } from '@/collab/useCollab'
import type { Peer } from '@/collab/useCollab'

const props = defineProps<{
  status: CollabStatus
  peers: Map<string, Peer>
}>()

const peerList = computed(() => Array.from(props.peers.values()))

const statusLabel = computed(() => {
  switch (props.status) {
    case 'connected': return 'Connected'
    case 'connecting': return 'Connecting…'
    case 'reconnecting': return 'Reconnecting…'
    default: return 'Disconnected'
  }
})
</script>

<style scoped>
.presence-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 12px;
  background: var(--color-surface, #1a1a2e);
  border-bottom: 1px solid var(--color-border, #2a2a3e);
  font-size: 12px;
  color: var(--color-text-secondary, #a0a0b0);
}

.presence-status {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.presence-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  display: inline-block;
}

.presence-dot--connected {
  background: #10b981;
  box-shadow: 0 0 4px #10b981;
}

.presence-dot--connecting {
  background: #f59e0b;
  animation: pulse 1s infinite;
}

.presence-dot--reconnecting {
  background: #f59e0b;
  animation: pulse 1s infinite;
}

.presence-dot--disconnected {
  background: #6b7280;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.presence-peers {
  display: flex;
  align-items: center;
  gap: 8px;
  overflow-x: auto;
}

.presence-peer {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.presence-avatar {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 600;
  color: #fff;
}

.presence-name {
  white-space: nowrap;
}
</style>
