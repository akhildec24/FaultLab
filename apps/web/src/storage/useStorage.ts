/**
 * useStorage — composable for local-first scenario storage.
 *
 * Provides:
 * - Auto-save: debounced save of graph state to IndexedDB
 * - History: snapshot history with restore capability
 * - Import/export: JSON download and upload
 * - Scenario list: list, load, delete saved scenarios
 * - Online/offline indicator
 */

import { ref, watch, onUnmounted, type Ref } from 'vue'
import {
  saveScenario,
  loadScenario,
  listScenarios,
  deleteScenario,
  saveHistoryEntry,
  loadHistory,
  clearHistory,
  generateScenarioId,
  exportScenarioJson,
  parseScenarioJson,
  type StoredScenario,
  type HistoryEntry,
} from './db'
import type { GraphNode, GraphEdge } from '@/graph/types'

const AUTOSAVE_DELAY = 2000
const HISTORY_INTERVAL = 30000

export function useStorage(
  nodes: Ref<GraphNode[]>,
  edges: Ref<GraphEdge[]>,
) {
  const currentScenarioId = ref<string | null>(null)
  const currentScenarioName = ref<string>('untitled')
  const savedScenarios = ref<StoredScenario[]>([])
  const history = ref<HistoryEntry[]>([])
  const isOnline = ref(navigator.onLine)
  const lastSaved = ref<number | null>(null)
  const isSaving = ref(false)

  let saveTimer: ReturnType<typeof setTimeout> | null = null
  let historyTimer: ReturnType<typeof setInterval> | null = null

  // --- Auto-save (debounced) ---

  function scheduleSave(): void {
    if (saveTimer) clearTimeout(saveTimer)
    saveTimer = setTimeout(doSave, AUTOSAVE_DELAY)
  }

  async function doSave(): Promise<void> {
    if (!currentScenarioId.value) {
      currentScenarioId.value = generateScenarioId()
    }
    isSaving.value = true
    try {
      const now = Date.now()
      const scenario: StoredScenario = {
        id: currentScenarioId.value,
        name: currentScenarioName.value,
        nodes: nodes.value,
        edges: edges.value,
        createdAt: lastSaved.value ?? now,
        updatedAt: now,
      }
      await saveScenario(scenario)
      lastSaved.value = now
    } catch (e) {
      console.error('Auto-save failed:', e)
    } finally {
      isSaving.value = false
    }
  }

  // Watch graph changes → schedule auto-save
  watch([nodes, edges], () => scheduleSave(), { deep: true })

  // --- History snapshots ---

  async function takeSnapshot(label?: string): Promise<void> {
    if (!currentScenarioId.value) return
    const entry: HistoryEntry = {
      scenarioId: currentScenarioId.value,
      timestamp: Date.now(),
      nodes: JSON.parse(JSON.stringify(nodes.value)),
      edges: JSON.parse(JSON.stringify(edges.value)),
      label: label ?? `Snapshot ${new Date().toLocaleTimeString()}`,
    }
    await saveHistoryEntry(entry)
    await refreshHistory()
  }

  async function restoreSnapshot(timestamp: number): Promise<void> {
    const entry = history.value.find((h) => h.timestamp === timestamp)
    if (!entry) return
    nodes.value = JSON.parse(JSON.stringify(entry.nodes)) as GraphNode[]
    edges.value = JSON.parse(JSON.stringify(entry.edges)) as GraphEdge[]
    scheduleSave()
  }

  async function refreshHistory(): Promise<void> {
    if (!currentScenarioId.value) {
      history.value = []
      return
    }
    try {
      history.value = await loadHistory(currentScenarioId.value)
    } catch (e) {
      console.error('Failed to load history:', e)
      history.value = []
    }
  }

  async function clearScenarioHistory(): Promise<void> {
    if (!currentScenarioId.value) return
    await clearHistory(currentScenarioId.value)
    history.value = []
  }

  // Start periodic history snapshots
  function startHistoryTimer(): void {
    if (historyTimer) clearInterval(historyTimer)
    historyTimer = setInterval(() => {
      if (nodes.value.length > 0) {
        takeSnapshot('Auto snapshot')
      }
    }, HISTORY_INTERVAL)
  }

  function stopHistoryTimer(): void {
    if (historyTimer) {
      clearInterval(historyTimer)
      historyTimer = null
    }
  }

  // --- Scenario management ---

  async function refreshScenarios(): Promise<void> {
    try {
      savedScenarios.value = await listScenarios()
    } catch (e) {
      console.error('Failed to list scenarios:', e)
      savedScenarios.value = []
    }
  }

  async function loadSavedScenario(id: string): Promise<void> {
    try {
      const scenario = await loadScenario(id)
      if (!scenario) return
      currentScenarioId.value = scenario.id
      currentScenarioName.value = scenario.name
      nodes.value = JSON.parse(JSON.stringify(scenario.nodes)) as GraphNode[]
      edges.value = JSON.parse(JSON.stringify(scenario.edges)) as GraphEdge[]
      lastSaved.value = scenario.updatedAt
      await refreshHistory()
    } catch (e) {
      console.error('Failed to load scenario:', e)
    }
  }

  async function deleteSavedScenario(id: string): Promise<void> {
    await deleteScenario(id)
    await refreshScenarios()
    if (currentScenarioId.value === id) {
      currentScenarioId.value = null
      history.value = []
    }
  }

  function newScenario(name?: string): void {
    currentScenarioId.value = null
    currentScenarioName.value = name ?? 'untitled'
    nodes.value = []
    edges.value = []
    history.value = []
    lastSaved.value = null
  }

  // --- Import / Export ---

  function exportJson(): string {
    const scenario: StoredScenario = {
      id: currentScenarioId.value ?? generateScenarioId(),
      name: currentScenarioName.value,
      nodes: JSON.parse(JSON.stringify(nodes.value)),
      edges: JSON.parse(JSON.stringify(edges.value)),
      createdAt: lastSaved.value ?? Date.now(),
      updatedAt: Date.now(),
    }
    return exportScenarioJson(scenario)
  }

  function downloadJson(): void {
    const json = exportJson()
    const blob = new Blob([json], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${currentScenarioName.value.replace(/\s+/g, '-')}.json`
    a.click()
    URL.revokeObjectURL(url)
  }

  async function importJson(json: string): Promise<void> {
    const scenario = parseScenarioJson(json)
    currentScenarioId.value = scenario.id
    currentScenarioName.value = scenario.name
    nodes.value = JSON.parse(JSON.stringify(scenario.nodes)) as GraphNode[]
    edges.value = JSON.parse(JSON.stringify(scenario.edges)) as GraphEdge[]
    lastSaved.value = scenario.updatedAt
    await doSave()
    await refreshScenarios()
    await refreshHistory()
  }

  async function importFromFile(file: File): Promise<void> {
    const text = await file.text()
    await importJson(text)
  }

  // --- Online/offline ---

  function handleOnline(): void {
    isOnline.value = true
  }
  function handleOffline(): void {
    isOnline.value = false
  }

  window.addEventListener('online', handleOnline)
  window.addEventListener('offline', handleOffline)

  // --- Lifecycle ---

  function init(): void {
    refreshScenarios()
    startHistoryTimer()
  }

  function destroy(): void {
    stopHistoryTimer()
    if (saveTimer) clearTimeout(saveTimer)
    window.removeEventListener('online', handleOnline)
    window.removeEventListener('offline', handleOffline)
  }

  onUnmounted(destroy)

  return {
    // State
    currentScenarioId,
    currentScenarioName,
    savedScenarios,
    history,
    isOnline,
    lastSaved,
    isSaving,
    // Auto-save
    save: doSave,
    // History
    takeSnapshot,
    restoreSnapshot,
    refreshHistory,
    clearScenarioHistory,
    // Scenario management
    refreshScenarios,
    loadSavedScenario,
    deleteSavedScenario,
    newScenario,
    // Import/export
    exportJson,
    downloadJson,
    importJson,
    importFromFile,
    // Lifecycle
    init,
    destroy,
  }
}
