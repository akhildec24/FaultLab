<script setup lang="ts">
import AppHeader from '@/components/AppHeader.vue'
import AppFooter from '@/components/AppFooter.vue'
</script>

<template>
  <div class="fl-app">
    <AppHeader />
    <main class="fl-main">
      <RouterView />
    </main>
    <AppFooter />
  </div>
</template>

<style scoped>
.fl-app {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

.fl-main {
  flex: 1;
}
</style>
