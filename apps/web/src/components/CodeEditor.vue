<script setup lang="ts">
/**
 * CodeEditor — a lightweight code editor for the FaultLab DSL.
 *
 * Features:
 * - Line numbers
 * - Syntax highlighting (keyword, string, number, duration, comment)
 * - Error underlining (from validateDsl)
 * - Tab inserts spaces (2)
 * - Visual-to-code sync: generates DSL from graph, can apply back
 */

import { computed, ref, watch } from 'vue'
import { useGraphStore } from '@/stores/graph'
import { graphToDsl, validateDsl, type DslError } from '@/graph/dsl'

const graph = useGraphStore()

const source = ref('')
const showLineNumbers = ref(true)
const syncMode = ref<'visual' | 'code'>('visual')

// Generate DSL from the visual graph
const generatedDsl = computed(() =>
  graphToDsl(graph.nodes, graph.edges, 'untitled', 42),
)

// Sync: when in visual mode, update source from graph
watch(
  generatedDsl,
  (dsl) => {
    if (syncMode.value === 'visual') {
      source.value = dsl
    }
  },
  { immediate: true },
)

// Validate the current source
const errors = computed<DslError[]>(() => validateDsl(source.value))

// Syntax highlighting — tokenise the source into HTML spans
const highlightedLines = computed(() => {
  const lines = source.value.split('\n')
  return lines.map((line) => highlightLine(line))
})

const KEYWORDS = new Set([
  'scenario', 'nodes', 'edges', 'traffic', 'failures', 'seed',
  'client', 'service', 'queue', 'cache', 'database', 'external_api',
  'name', 'capacity', 'latency', 'error_rate', 'timeout',
  'queue_limit', 'cache_hit_rate', 'replication', 'replication_lag',
  'retry', 'shed', 'immediate', 'fixed', 'exponential',
  'drop', 'reject', 'backpressure', 'standalone', 'leader', 'replica',
  'start', 'target', 'ramp', 'at', 'rps',
  'crash', 'recover', 'add_latency', 'disconnect',
  'add_packet_loss', 'reduce_capacity',
  'max_retries', 'jitter', 'delay', 'base', 'max', 'budget',
  'packet_loss', 'bandwidth',
])

function highlightLine(line: string): string {
  let result = ''
  let i = 0
  while (i < line.length) {
    const ch = line[i]

    // Comments
    if (ch === '#') {
      result += `<span class="tok-comment">${escapeHtml(line.slice(i))}</span>`
      break
    }
    if (ch === '/' && line[i + 1] === '/') {
      result += `<span class="tok-comment">${escapeHtml(line.slice(i))}</span>`
      break
    }

    // Strings
    if (ch === '"') {
      let end = i + 1
      while (end < line.length && line[end] !== '"') end++
      const str = line.slice(i, end + 1)
      result += `<span class="tok-string">${escapeHtml(str)}</span>`
      i = end + 1
      continue
    }

    // Numbers with duration/percent suffixes
    if (ch >= '0' && ch <= '9') {
      let end = i
      while (end < line.length && ((line[end] >= '0' && line[end] <= '9') || line[end] === '.')) end++
      // Check for duration suffix
      const rest = line.slice(end)
      if (rest.startsWith('ms')) {
        result += `<span class="tok-duration">${escapeHtml(line.slice(i, end + 2))}</span>`
        i = end + 2
        continue
      }
      if (rest.startsWith('s') && !rest.startsWith('seed')) {
        result += `<span class="tok-duration">${escapeHtml(line.slice(i, end + 1))}</span>`
        i = end + 1
        continue
      }
      if (rest.startsWith('%')) {
        result += `<span class="tok-percent">${escapeHtml(line.slice(i, end + 1))}</span>`
        i = end + 1
        continue
      }
      result += `<span class="tok-number">${escapeHtml(line.slice(i, end))}</span>`
      i = end
      continue
    }

    // Identifiers / keywords
    if (ch >= 'a' && ch <= 'z' || ch >= 'A' && ch <= 'Z' || ch === '_') {
      let end = i
      while (end < line.length && ((line[end] >= 'a' && line[end] <= 'z') || (line[end] >= 'A' && line[end] <= 'Z') || line[end] === '_' || (line[end] >= '0' && line[end] <= '9'))) end++
      const word = line.slice(i, end)
      if (KEYWORDS.has(word)) {
        result += `<span class="tok-keyword">${escapeHtml(word)}</span>`
      } else {
        result += escapeHtml(word)
      }
      i = end
      continue
    }

    // Arrow
    if (ch === '-' && line[i + 1] === '>') {
      result += `<span class="tok-arrow">-&gt;</span>`
      i += 2
      continue
    }

    // Braces
    if (ch === '{' || ch === '}') {
      result += `<span class="tok-brace">${escapeHtml(ch)}</span>`
      i++
      continue
    }

    // Default
    result += escapeHtml(ch)
    i++
  }
  return result
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
}

// Error lines for underlining
const errorLines = computed(() => {
  const map = new Map<number, DslError[]>()
  for (const err of errors.value) {
    const existing = map.get(err.line) || []
    existing.push(err)
    map.set(err.line, existing)
  }
  return map
})

// Line count for line numbers
const lineCount = computed(() => source.value.split('\n').length)

// Handle textarea input
function onInput(e: Event): void {
  syncMode.value = 'code'
  source.value = (e.target as HTMLTextAreaElement).value
}

// Handle tab key — insert 2 spaces
function onKeydown(e: KeyboardEvent): void {
  if (e.key === 'Tab') {
    e.preventDefault()
    const ta = e.target as HTMLTextAreaElement
    const start = ta.selectionStart
    const end = ta.selectionEnd
    const newSource = source.value.slice(0, start) + '  ' + source.value.slice(end)
    source.value = newSource
    // Restore cursor position
    requestAnimationFrame(() => {
      ta.selectionStart = ta.selectionEnd = start + 2
    })
  }
}

// Switch back to visual sync mode
function syncFromVisual(): void {
  syncMode.value = 'visual'
  source.value = generatedDsl.value
}

// Copy DSL to clipboard
async function copyDsl(): Promise<void> {
  try {
    await navigator.clipboard.writeText(source.value)
  } catch {
    // Ignore clipboard errors
  }
}
</script>

<template>
  <div class="code-editor">
    <div class="code-editor__toolbar">
      <span class="code-editor__mode" :class="{ 'is-code': syncMode === 'code' }">
        {{ syncMode === 'visual' ? '🔗 Synced from visual' : '✏️ Edited (unsynced)' }}
      </span>
      <button
        v-if="syncMode === 'code'"
        class="code-editor__btn"
        @click="syncFromVisual"
      >
        ↻ Re-sync from visual
      </button>
      <button class="code-editor__btn" @click="copyDsl">
        ⧉ Copy
      </button>
    </div>

    <!-- Errors -->
    <div class="code-editor__errors" v-if="errors.length > 0">
      <div class="code-editor__error" v-for="(err, i) in errors" :key="i">
        ⚠ Line {{ err.line }}: {{ err.message }}
      </div>
    </div>

    <!-- Editor body -->
    <div class="code-editor__body">
      <!-- Line numbers -->
      <div class="code-editor__gutter" v-if="showLineNumbers">
        <span
          v-for="n in lineCount"
          :key="n"
          class="code-editor__line-num"
          :class="{ 'has-error': errorLines.has(n) }"
        >{{ n }}</span>
      </div>

      <!-- Highlighted code (display layer) -->
      <pre class="code-editor__highlight" aria-hidden="true"><code
        v-for="(line, i) in highlightedLines"
        :key="i"
        :class="{ 'code-editor__error-line': errorLines.has(i + 1) }"
      >{{ line || ' ' }}<br /></code></pre>

      <!-- Textarea (input layer, transparent text) -->
      <textarea
        class="code-editor__textarea"
        :value="source"
        @input="onInput"
        @keydown="onKeydown"
        spellcheck="false"
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
      ></textarea>
    </div>
  </div>
</template>

<style scoped>
.code-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background: var(--fl-bg);
  font-family: var(--fl-font-mono);
}

.code-editor__toolbar {
  display: flex;
  align-items: center;
  gap: var(--fl-space-2);
  padding: var(--fl-space-1) var(--fl-space-2);
  border-bottom: 1px solid var(--fl-border);
  flex-shrink: 0;
}

.code-editor__mode {
  font-size: var(--fl-size-14);
  color: var(--fl-text-secondary);
}

.code-editor__mode.is-code {
  color: var(--fl-amber);
}

.code-editor__btn {
  margin-left: auto;
  padding: 2px 8px;
  background: transparent;
  border: 1px solid var(--fl-border);
  border-radius: 4px;
  color: var(--fl-text-secondary);
  font-size: var(--fl-size-14);
  cursor: pointer;
}

.code-editor__btn:hover {
  color: var(--fl-amber);
  border-color: var(--fl-amber);
}

.code-editor__errors {
  padding: var(--fl-space-1) var(--fl-space-2);
  background: var(--fl-red-light);
  border-bottom: 1px solid var(--fl-red);
  flex-shrink: 0;
  max-height: 120px;
  overflow-y: auto;
}

.code-editor__error {
  font-size: var(--fl-size-14);
  color: var(--fl-red);
  font-family: var(--fl-font-mono);
}

.code-editor__body {
  flex: 1;
  position: relative;
  overflow: hidden;
  display: flex;
}

.code-editor__gutter {
  display: flex;
  flex-direction: column;
  padding: var(--fl-space-2) var(--fl-space-1);
  background: var(--fl-bg-alt);
  border-right: 1px solid var(--fl-border);
  text-align: right;
  flex-shrink: 0;
  user-select: none;
  overflow: hidden;
}

.code-editor__line-num {
  font-size: 0.75rem;
  color: var(--fl-grey-3);
  line-height: 1.4;
  font-family: var(--fl-font-mono);
}

.code-editor__line-num.has-error {
  color: var(--fl-red);
  font-weight: 700;
}

.code-editor__highlight {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  padding: var(--fl-space-2);
  margin: 0;
  font-family: var(--fl-font-mono);
  font-size: 0.8125rem;
  line-height: 1.4;
  white-space: pre-wrap;
  word-break: break-word;
  overflow: auto;
  pointer-events: none;
  color: var(--fl-text);
}

.code-editor__error-line {
  background: rgba(220, 38, 38, 0.1);
  text-decoration: underline wavy var(--fl-red);
}

.code-editor__textarea {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  padding: var(--fl-space-2);
  border: none;
  outline: none;
  resize: none;
  font-family: var(--fl-font-mono);
  font-size: 0.8125rem;
  line-height: 1.4;
  white-space: pre-wrap;
  word-break: break-word;
  background: transparent;
  color: transparent;
  caret-color: var(--fl-text);
  overflow: auto;
}

.code-editor__textarea::selection {
  background: rgba(245, 158, 11, 0.3);
}

/* Syntax highlighting tokens */
:deep(.tok-keyword) {
  color: #6366f1;
  font-weight: 600;
}
:deep(.tok-string) {
  color: #059669;
}
:deep(.tok-number) {
  color: #d97706;
}
:deep(.tok-duration) {
  color: #0ea5e9;
}
:deep(.tok-percent) {
  color: #dc2626;
}
:deep(.tok-comment) {
  color: var(--fl-grey-3);
  font-style: italic;
}
:deep(.tok-brace) {
  color: var(--fl-text);
  font-weight: 700;
}
:deep(.tok-arrow) {
  color: var(--fl-amber);
  font-weight: 700;
}
</style>
