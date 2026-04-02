<script setup lang="ts">
import { RouterLink } from 'vue-router'
import { FlaskConical } from '@lucide/vue'
</script>

<template>
  <header class="fl-header">
    <div class="fl-header__bar">
      <div class="fl-container fl-header__inner">
        <RouterLink to="/" class="fl-header__brand">
          <FlaskConical class="fl-header__logo" :size="22" aria-hidden="true" />
          <span class="fl-header__title">FaultLab</span>
        </RouterLink>
        <nav class="fl-header__nav" aria-label="Primary">
          <RouterLink to="/" class="fl-header__link">Home</RouterLink>
          <RouterLink to="/editor" class="fl-header__link">Editor</RouterLink>
          <RouterLink to="/architecture" class="fl-header__link">Architecture</RouterLink>
          <RouterLink to="/docs" class="fl-header__link">Docs</RouterLink>
        </nav>
      </div>
    </div>
    <div class="fl-header__phase">
      <div class="fl-container">
        <span class="fl-tag fl-tag--yellow">Alpha</span>
        <span class="fl-header__phase-text">This is a development tool — feedback is welcome.</span>
      </div>
    </div>
  </header>
</template>

<style scoped>
.fl-header__bar {
  background: var(--fl-slate);
  color: var(--fl-white);
}

.fl-header__inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 56px;
  padding-top: var(--fl-space-2);
  padding-bottom: var(--fl-space-2);
}

.fl-header__brand {
  display: flex;
  align-items: center;
  gap: var(--fl-space-2);
  color: var(--fl-white);
  text-decoration: none;
  font-size: var(--fl-size-24);
  font-weight: 700;
}

.fl-header__brand:hover {
  color: var(--fl-white);
}

.fl-header__logo {
  color: var(--fl-amber);
  flex-shrink: 0;
}

.fl-header__nav {
  display: flex;
  gap: var(--fl-space-4);
}

.fl-header__link {
  color: var(--fl-white);
  text-decoration: none;
  font-size: var(--fl-size-19);
  font-weight: 400;
  padding: var(--fl-space-1) 0;
  border-bottom: 3px solid transparent;
  transition: border-color 0.1s ease;
}

.fl-header__link:hover {
  color: var(--fl-amber);
  border-bottom-color: var(--fl-amber);
}

.router-link-active.fl-header__link {
  border-bottom-color: var(--fl-amber);
  color: var(--fl-amber);
}

.fl-header__phase {
  background: var(--fl-bg-alt);
  border-bottom: 1px solid var(--fl-border);
  padding: var(--fl-space-2) 0;
}

.fl-header__phase .fl-container {
  display: flex;
  align-items: center;
  gap: var(--fl-space-2);
}

.fl-header__phase-text {
  font-size: var(--fl-size-16);
  color: var(--fl-grey-4);
}

@media (max-width: 640px) {
  .fl-header__inner {
    flex-direction: column;
    align-items: flex-start;
    gap: var(--fl-space-2);
  }

  .fl-header__nav {
    flex-wrap: wrap;
    gap: var(--fl-space-3);
  }
}
</style>
