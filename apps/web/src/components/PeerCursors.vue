<template>
  <div class="peer-cursors" v-if="peers.size > 0">
    <div
      v-for="peer in peerList"
      :key="peer.id"
      v-show="peer.cursor"
      class="peer-cursor"
      :style="{
        left: peer.cursor?.x + 'px',
        top: peer.cursor?.y + 'px',
        color: peer.color,
      }"
    >
      <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
        <path d="M0 0 L0 14 L4 10 L7 16 L9 15 L6 9 L12 9 Z" />
      </svg>
      <span class="peer-cursor-label" :style="{ background: peer.color }">
        {{ peer.name }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Peer } from '@/collab/useCollab'

const props = defineProps<{
  peers: Map<string, Peer>
}>()

const peerList = computed(() => Array.from(props.peers.values()))
</script>

<style scoped>
.peer-cursors {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 1000;
}

.peer-cursor {
  position: absolute;
  transition: left 0.1s ease-out, top 0.1s ease-out;
  pointer-events: none;
}

.peer-cursor-label {
  position: absolute;
  top: 16px;
  left: 12px;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
  color: #fff;
  white-space: nowrap;
}
</style>
