/**
 * IndexedDB wrapper for local-first storage of graph scenarios.
 *
 * Stores named scenarios with full graph state, timestamps, and
 * history snapshots. Works offline — no network required.
 *
 * Database: faultlab-db
 * Object stores:
 *   - scenarios: key=id, value=StoredScenario
 *   - history: key=id+timestamp, value=HistoryEntry
 */

const DB_NAME = 'faultlab-db'
const DB_VERSION = 1
const SCENARIOS_STORE = 'scenarios'
const HISTORY_STORE = 'history'

export interface StoredScenario {
  id: string
  name: string
  nodes: unknown[]
  edges: unknown[]
  createdAt: number
  updatedAt: number
}

export interface HistoryEntry {
  scenarioId: string
  timestamp: number
  nodes: unknown[]
  edges: unknown[]
  label: string
}

let dbPromise: Promise<IDBDatabase> | null = null

function openDb(): Promise<IDBDatabase> {
  if (dbPromise) return dbPromise

  dbPromise = new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION)

    request.onupgradeneeded = () => {
      const db = request.result
      if (!db.objectStoreNames.contains(SCENARIOS_STORE)) {
        db.createObjectStore(SCENARIOS_STORE, { keyPath: 'id' })
      }
      if (!db.objectStoreNames.contains(HISTORY_STORE)) {
        const store = db.createObjectStore(HISTORY_STORE, { keyPath: ['scenarioId', 'timestamp'] })
        store.createIndex('byScenario', 'scenarioId', { unique: false })
      }
    }

    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error)
  })

  return dbPromise
}

function tx<T>(
  storeName: string,
  mode: IDBTransactionMode,
  fn: (store: IDBObjectStore) => IDBRequest<T>,
): Promise<T> {
  return openDb().then(
    (db) =>
      new Promise<T>((resolve, reject) => {
        const transaction = db.transaction(storeName, mode)
        const store = transaction.objectStore(storeName)
        const request = fn(store)
        request.onsuccess = () => resolve(request.result)
        request.onerror = () => reject(request.error)
      }),
  )
}

// --- Scenario CRUD ---

export async function saveScenario(scenario: StoredScenario): Promise<void> {
  await tx<IDBValidKey>(SCENARIOS_STORE, 'readwrite', (store) => store.put(scenario))
}

export async function loadScenario(id: string): Promise<StoredScenario | undefined> {
  return tx<StoredScenario>(SCENARIOS_STORE, 'readonly', (store) =>
    store.get(id) as IDBRequest<StoredScenario>,
  )
}

export async function listScenarios(): Promise<StoredScenario[]> {
  return tx<StoredScenario[]>(SCENARIOS_STORE, 'readonly', (store) =>
    store.getAll() as IDBRequest<StoredScenario[]>,
  )
}

export async function deleteScenario(id: string): Promise<void> {
  await tx<undefined>(SCENARIOS_STORE, 'readwrite', (store) => store.delete(id))
  // Also delete history entries
  const db = await openDb()
  const transaction = db.transaction(HISTORY_STORE, 'readwrite')
  const store = transaction.objectStore(HISTORY_STORE)
  const index = store.index('byScenario')
  const range = IDBKeyRange.only(id)
  index.openCursor(range).onsuccess = (event) => {
    const cursor = (event.target as IDBRequest<IDBCursorWithValue>).result
    if (cursor) {
      cursor.delete()
      cursor.continue()
    }
  }
}

// --- History ---

export async function saveHistoryEntry(entry: HistoryEntry): Promise<void> {
  await tx<IDBValidKey>(HISTORY_STORE, 'readwrite', (store) => store.put(entry))
}

export async function loadHistory(scenarioId: string): Promise<HistoryEntry[]> {
  const db = await openDb()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(HISTORY_STORE, 'readonly')
    const store = transaction.objectStore(HISTORY_STORE)
    const index = store.index('byScenario')
    const range = IDBKeyRange.only(scenarioId)
    const request = index.getAll(range)
    request.onsuccess = () => {
      const results = request.result as HistoryEntry[]
      results.sort((a, b) => b.timestamp - a.timestamp)
      resolve(results)
    }
    request.onerror = () => reject(request.error)
  })
}

export async function clearHistory(scenarioId: string): Promise<void> {
  const db = await openDb()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(HISTORY_STORE, 'readwrite')
    const store = transaction.objectStore(HISTORY_STORE)
    const index = store.index('byScenario')
    const range = IDBKeyRange.only(scenarioId)
    const cursorRequest = index.openCursor(range)
    cursorRequest.onsuccess = (event) => {
      const cursor = (event.target as IDBRequest<IDBCursorWithValue>).result
      if (cursor) {
        cursor.delete()
        cursor.continue()
      }
    }
    transaction.oncomplete = () => resolve()
    transaction.onerror = () => reject(transaction.error)
  })
}

// --- Utility ---

export function generateScenarioId(): string {
  return `scenario-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
}

export function exportScenarioJson(scenario: StoredScenario): string {
  return JSON.stringify(scenario, null, 2)
}

export function parseScenarioJson(json: string): StoredScenario {
  const parsed = JSON.parse(json)
  if (!parsed.id || !parsed.name || !Array.isArray(parsed.nodes)) {
    throw new Error('Invalid scenario JSON: missing required fields')
  }
  return parsed as StoredScenario
}
