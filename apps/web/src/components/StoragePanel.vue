<script setup lang="ts">
/**
 * StoragePanel — UI for local-first scenario management.
 *
 * Shows saved scenarios, history snapshots, import/export buttons,
 * and online/offline status. Designed as a sidebar or dropdown.
 */

import { ref, onMounted } from 'vue'
import { useGraphStore } from '@/stores/graph'
import { useStorage } from '@/storage/useStorage'
import {
  Wifi,
  WifiOff,
  FilePlus,
  Save,
  Download,
  Upload,
  Camera,
  ChevronDown,
  ChevronRight,
  X,
} from '@lucide/vue'

const graph = useGraphStore()
const storage = useStorage(graph.nodes, graph.edges)

const showHistory = ref(false)
const showScenarios = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

onMounted(() => {
  storage.init()
})

function handleFileUpload(e: Event): void {
  const target = e.target as HTMLInputElement
  if (target.files && target.files[0]) {
    storage.importFromFile(target.files[0])
    target.value = ''
  }
}

function formatDate(ts: number): string {
  const d = new Date(ts)
  return d.toLocaleDateString() + ' ' + d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

function newScenario(): void {
  if (confirm('Create new scenario? Unsaved changes will be lost.')) {
    graph.clear()
    storage.newScenario('untitled')
  }
}
</script>

<template>
  <div class="storage-panel">
    <!-- Status bar -->
    <div class="storage-panel__status">
      <span class="storage-panel__indicator" :class="{ 'is-offline': !storage.isOnline.value }">
        <Wifi v-if="storage.isOnline.value" :size="14" />
        <WifiOff v-else :size="14" />
        {{ storage.isOnline.value ? 'Online' : 'Offline' }}
      </span>
      <span class="storage-panel__scenario-name" v-if="storage.currentScenarioName.value">
        {{ storage.currentScenarioName.value }}
      </span>
      <span class="storage-panel__saved" v-if="storage.lastSaved.value">
        Saved {{ formatDate(storage.lastSaved.value) }}
      </span>
      <span class="storage-panel__saving" v-if="storage.isSaving.value">
        Saving…
      </span>
    </div>

    <!-- Action buttons -->
    <div class="storage-panel__actions">
      <button class="storage-btn" @click="newScenario" title="New scenario">
        <FilePlus :size="16" /> New
      </button>
      <button class="storage-btn" @click="storage.save()" title="Save now">
        <Save :size="16" /> Save
      </button>
      <button class="storage-btn" @click="storage.downloadJson()" title="Export JSON">
        <Download :size="16" /> Export
      </button>
      <button class="storage-btn" @click="fileInput?.click()" title="Import JSON">
        <Upload :size="16" /> Import
      </button>
      <input
        ref="fileInput"
        type="file"
        accept=".json"
        style="display: none"
        @change="handleFileUpload"
      />
      <button class="storage-btn" @click="storage.takeSnapshot('Manual snapshot')" title="Take snapshot">
        <Camera :size="16" /> Snapshot
      </button>
    </div>

    <!-- Collapsible sections -->
    <div class="storage-panel__sections">
      <!-- Saved scenarios -->
      <div class="storage-section">
        <button
          class="storage-section__header"
          @click="showScenarios = !showScenarios"
        >
          <ChevronDown v-if="showScenarios" :size="16" />
        <ChevronRight v-else :size="16" />
        Saved Scenarios ({{ storage.savedScenarios.value.length }})
        </button>
        <div class="storage-section__body" v-if="showScenarios">
          <div
            v-for="s in storage.savedScenarios.value"
            :key="s.id"
            class="storage-item"
            :class="{ 'is-current': s.id === storage.currentScenarioId.value }"
          >
            <div class="storage-item__info" @click="storage.loadSavedScenario(s.id)">
              <span class="storage-item__name">{{ s.name }}</span>
              <span class="storage-item__date">{{ formatDate(s.updatedAt) }}</span>
            </div>
            <button
              class="storage-item__delete"
              @click.stop="storage.deleteSavedScenario(s.id)"
              title="Delete"
            >
              <X :size="16" />
            </button>
          </div>
          <div class="storage-empty" v-if="storage.savedScenarios.value.length === 0">
            No saved scenarios yet.
          </div>
        </div>
      </div>

      <!-- History -->
      <div class="storage-section">
        <button
          class="storage-section__header"
          @click="showHistory = !showHistory"
        >
          <ChevronDown v-if="showHistory" :size="16" />
        <ChevronRight v-else :size="16" />
        History ({{ storage.history.value.length }})
        </button>
        <div class="storage-section__body" v-if="showHistory">
          <div
            v-for="h in storage.history.value.slice(0, 20)"
            :key="h.timestamp"
            class="storage-item"
          >
            <div class="storage-item__info" @click="storage.restoreSnapshot(h.timestamp)">
              <span class="storage-item__name">{{ h.label }}</span>
              <span class="storage-item__date">{{ formatDate(h.timestamp) }}</span>
            </div>
          </div>
          <div class="storage-empty" v-if="storage.history.value.length === 0">
            No history snapshots yet.
          </div>
          <button
            class="storage-btn storage-btn--danger"
            v-if="storage.history.value.length > 0"
            @click="storage.clearScenarioHistory()"
          >
            Clear History
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.storage-panel {
  display: flex;
  flex-direction: column;
  gap: var(--fl-space-2);
  padding: var(--fl-space-2);
  font-size: var(--fl-size-14);
}

.storage-panel__status {
  display: flex;
  align-items: center;
  gap: var(--fl-space-2);
  flex-wrap: wrap;
}

.storage-panel__indicator {
  color: var(--fl-green);
  font-size: 0.75rem;
}

.storage-panel__indicator.is-offline {
  color: var(--fl-red);
}

.storage-panel__scenario-name {
  font-weight: 600;
  color: var(--fl-text);
}

.storage-panel__saved {
  color: var(--fl-text-secondary);
  font-size: 0.75rem;
}

.storage-panel__saving {
  color: var(--fl-amber);
  font-size: 0.75rem;
}

.storage-panel__actions {
  display: flex;
  gap: var(--fl-space-1);
  flex-wrap: wrap;
}

.storage-btn {
  padding: 4px 10px;
  background: transparent;
  border: 1px solid var(--fl-border);
  border-radius: 4px;
  color: var(--fl-text-secondary);
  font-size: var(--fl-size-14);
  cursor: pointer;
  white-space: nowrap;
}

.storage-btn:hover {
  color: var(--fl-amber);
  border-color: var(--fl-amber);
}

.storage-btn--danger:hover {
  color: var(--fl-red);
  border-color: var(--fl-red);
}

.storage-panel__sections {
  display: flex;
  flex-direction: column;
  gap: var(--fl-space-1);
}

.storage-section__header {
  width: 100%;
  text-align: left;
  padding: var(--fl-space-1) var(--fl-space-2);
  background: transparent;
  border: 1px solid var(--fl-border);
  border-radius: 4px;
  color: var(--fl-text-secondary);
  font-size: var(--fl-size-14);
  cursor: pointer;
}

.storage-section__header:hover {
  color: var(--fl-text);
}

.storage-section__body {
  padding: var(--fl-space-1);
  display: flex;
  flex-direction: column;
  gap: var(--fl-space-1);
}

.storage-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--fl-space-1) var(--fl-space-2);
  border: 1px solid transparent;
  border-radius: 4px;
  cursor: pointer;
}

.storage-item:hover {
  border-color: var(--fl-border);
  background: var(--fl-bg-alt);
}

.storage-item.is-current {
  border-color: var(--fl-amber);
  background: rgba(245, 158, 11, 0.05);
}

.storage-item__info {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
}

.storage-item__name {
  color: var(--fl-text);
  font-size: var(--fl-size-14);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.storage-item__date {
  color: var(--fl-text-secondary);
  font-size: 0.75rem;
}

.storage-item__delete {
  padding: 2px 6px;
  background: transparent;
  border: none;
  color: var(--fl-text-secondary);
  cursor: pointer;
  font-size: var(--fl-size-14);
}

.storage-item__delete:hover {
  color: var(--fl-red);
}

.storage-empty {
  color: var(--fl-text-secondary);
  font-size: 0.75rem;
  padding: var(--fl-space-1);
  text-align: center;
}
</style>
