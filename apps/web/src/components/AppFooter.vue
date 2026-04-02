<script setup lang="ts">
const year = new Date().getFullYear()
</script>

<template>
  <footer class="fl-footer">
    <div class="fl-container">
      <div class="fl-footer__meta">
        <div class="fl-footer__meta-item">
          FaultLab is an open-source educational tool for distributed systems simulation.
        </div>
        <div class="fl-footer__meta-item fl-footer__meta-item--link">
          <a href="https://github.com/akhildec24/FaultLab" class="fl-footer__link">Source code</a>
        </div>
      </div>
      <div class="fl-footer__copyright">
        © {{ year }} FaultLab. Built as a technical demonstration.
      </div>
    </div>
  </footer>
</template>

<style scoped>
.fl-footer {
  background: var(--fl-bg-alt);
  border-top: 1px solid var(--fl-border);
  padding: var(--fl-space-5) 0;
  margin-top: var(--fl-space-7);
}

.fl-footer__meta {
  display: flex;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: var(--fl-space-3);
  padding-bottom: var(--fl-space-4);
  border-bottom: 1px solid var(--fl-border);
}

.fl-footer__meta-item {
  font-size: var(--fl-size-16);
  color: var(--fl-grey-4);
  max-width: 480px;
}

.fl-footer__link {
  font-size: var(--fl-size-16);
}

.fl-footer__copyright {
  padding-top: var(--fl-space-3);
  font-size: var(--fl-size-14);
  color: var(--fl-grey-3);
}
</style>
