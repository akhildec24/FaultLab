<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import AppHeader from '@/components/AppHeader.vue'
import AppFooter from '@/components/AppFooter.vue'

const route = useRoute()
const isEditor = computed(() => route.name === 'editor')
const showFooter = computed(() => !isEditor.value)
</script>

<template>
  <div :class="['fl-app', { 'fl-app--full': isEditor }]">
    <AppHeader />
    <main class="fl-main">
      <RouterView />
    </main>
    <AppFooter v-if="showFooter" />
  </div>
</template>

<style scoped>
.fl-app {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

.fl-app--full {
  height: 100vh;
  overflow: hidden;
}

.fl-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
</style>
