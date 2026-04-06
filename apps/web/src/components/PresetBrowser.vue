<script setup lang="ts">
/**
 * PresetBrowser — card-based preset scenario picker.
 *
 * Shows a grid of preset cards with names, descriptions, node counts,
 * and category badges. Clicking a card loads the preset.
 */

import { ref, computed } from 'vue'
import { useGraphStore } from '@/stores/graph'
import { useSimulationStore } from '@/stores/simulation'
import { useAnimationStore } from '@/stores/animation'
import { PRESETS, type PresetScenario } from '@/graph/presets'
import { X, Search, ArrowRight } from '@lucide/vue'

const emit = defineEmits<{ close: [] }>()

const graph = useGraphStore()
const sim = useSimulationStore()
const animation = useAnimationStore()

const searchQuery = ref('')
const activeCategory = ref('All')

const categories = computed(() => {
  const cats = new Set<string>()
  PRESETS.forEach((p) => {
    if (p.category) cats.add(p.category)
  })
  return ['All', ...Array.from(cats).sort()]
})

const filteredPresets = computed(() => {
  let list = PRESETS
  if (activeCategory.value !== 'All') {
    list = list.filter((p) => p.category === activeCategory.value)
  }
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    list = list.filter(
      (p) =>
        p.name.toLowerCase().includes(q) ||
        p.description.toLowerCase().includes(q),
    )
  }
  return list
})

async function loadPreset(preset: PresetScenario): Promise<void> {
  animation.clear()
  try {
    await sim.pause()
    await sim.reset()
  } catch {
    // Ignore
  }
  graph.loadPreset(preset)
  emit('close')
}

function nodeCount(p: PresetScenario): number {
  return p.nodes.length
}

function edgeCount(p: PresetScenario): number {
  return p.connections.length
}
</script>

<template>
  <div class="preset-browser__overlay" @click.self="emit('close')">
    <div class="preset-browser">
      <!-- Header -->
      <div class="preset-browser__header">
        <h2 class="preset-browser__title">Preset Scenarios</h2>
        <button class="preset-browser__close" @click="emit('close')">
          <X :size="20" />
        </button>
      </div>

      <!-- Search + filters -->
      <div class="preset-browser__filters">
        <div class="preset-browser__search">
          <Search :size="16" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search scenarios..."
            class="preset-browser__search-input"
          />
        </div>
        <div class="preset-browser__categories">
          <button
            v-for="cat in categories"
            :key="cat"
            :class="[
              'preset-browser__category',
              { 'preset-browser__category--active': activeCategory === cat },
            ]"
            @click="activeCategory = cat"
          >
            {{ cat }}
          </button>
        </div>
      </div>

      <!-- Grid -->
      <div class="preset-browser__grid">
        <button
          v-for="p in filteredPresets"
          :key="p.id"
          class="preset-card"
          @click="loadPreset(p)"
        >
          <div class="preset-card__top">
            <span v-if="p.category" class="preset-card__category">{{ p.category }}</span>
            <span class="preset-card__stats">{{ nodeCount(p) }} nodes, {{ edgeCount(p) }} edges</span>
          </div>
          <h3 class="preset-card__name">{{ p.name }}</h3>
          <p class="preset-card__desc">{{ p.description }}</p>
          <div class="preset-card__footer">
            <span class="preset-card__cta">Load scenario <ArrowRight :size="14" /></span>
          </div>
        </button>
      </div>

      <div v-if="filteredPresets.length === 0" class="preset-browser__empty">
        No scenarios match your search.
      </div>
    </div>
  </div>
</template>

<style scoped>
.preset-browser__overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--fl-space-4);
}

.preset-browser {
  background: var(--fl-white);
  border-radius: 8px;
  max-width: 900px;
  width: 100%;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.preset-browser__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--fl-space-4) var(--fl-space-5);
  border-bottom: 1px solid var(--fl-border);
}

.preset-browser__title {
  font-size: var(--fl-size-27);
  color: var(--fl-text);
}

.preset-browser__close {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--fl-grey-3);
  padding: var(--fl-space-2);
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.preset-browser__close:hover {
  background: var(--fl-bg-alt);
  color: var(--fl-text);
}

.preset-browser__filters {
  padding: var(--fl-space-3) var(--fl-space-5);
  border-bottom: 1px solid var(--fl-border);
  display: flex;
  flex-direction: column;
  gap: var(--fl-space-2);
}

.preset-browser__search {
  display: flex;
  align-items: center;
  gap: var(--fl-space-2);
  background: var(--fl-bg-alt);
  border: 1px solid var(--fl-border);
  border-radius: 6px;
  padding: var(--fl-space-2) var(--fl-space-3);
  color: var(--fl-grey-3);
}

.preset-browser__search-input {
  flex: 1;
  border: none;
  background: none;
  font-family: var(--fl-font);
  font-size: var(--fl-size-16);
  color: var(--fl-text);
  outline: none;
}

.preset-browser__categories {
  display: flex;
  gap: var(--fl-space-1);
  flex-wrap: wrap;
}

.preset-browser__category {
  font-family: var(--fl-font);
  font-size: var(--fl-size-14);
  padding: var(--fl-space-1) var(--fl-space-3);
  border: 1px solid var(--fl-border);
  border-radius: 4px;
  background: var(--fl-white);
  color: var(--fl-grey-4);
  cursor: pointer;
  transition: all var(--fl-transition);
}

.preset-browser__category:hover {
  border-color: var(--fl-amber);
  color: var(--fl-text);
}

.preset-browser__category--active {
  background: var(--fl-slate);
  color: var(--fl-white);
  border-color: var(--fl-slate);
}

.preset-browser__grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: var(--fl-space-3);
  padding: var(--fl-space-4) var(--fl-space-5);
  overflow-y: auto;
  flex: 1;
}

.preset-card {
  display: flex;
  flex-direction: column;
  text-align: left;
  background: var(--fl-white);
  border: 1px solid var(--fl-border);
  border-radius: 6px;
  padding: var(--fl-space-3);
  cursor: pointer;
  transition: border-color var(--fl-transition), box-shadow var(--fl-transition);
  font-family: var(--fl-font);
  color: inherit;
}

.preset-card:hover {
  border-color: var(--fl-amber);
  box-shadow: var(--fl-shadow-md);
}

.preset-card__top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--fl-space-2);
}

.preset-card__category {
  font-size: var(--fl-size-14);
  font-weight: 600;
  color: var(--fl-amber-hover);
  background: var(--fl-amber-light);
  padding: 2px 8px;
  border-radius: 4px;
}

.preset-card__stats {
  font-size: var(--fl-size-14);
  color: var(--fl-grey-3);
}

.preset-card__name {
  font-size: var(--fl-size-19);
  color: var(--fl-text);
  margin-bottom: var(--fl-space-1);
}

.preset-card__desc {
  font-size: var(--fl-size-14);
  color: var(--fl-grey-4);
  line-height: 1.5;
  flex: 1;
  margin-bottom: var(--fl-space-2);
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.preset-card__footer {
  display: flex;
  align-items: center;
}

.preset-card__cta {
  font-size: var(--fl-size-14);
  font-weight: 600;
  color: var(--fl-amber-hover);
  display: flex;
  align-items: center;
  gap: 4px;
}

.preset-browser__empty {
  text-align: center;
  padding: var(--fl-space-6);
  color: var(--fl-grey-3);
  font-size: var(--fl-size-16);
}
</style>
