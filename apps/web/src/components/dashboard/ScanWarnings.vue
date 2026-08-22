<script setup lang="ts">
import type { ScanResult } from '@/types/scan'

defineProps<{
  result: ScanResult
}>()
</script>

<template>
  <section
    v-if="result.errors.length > 0"
    class="scan-warnings"
    role="status"
    aria-live="polite"
  >
    <div class="scan-warnings__header">
      <span class="scan-warnings__icon" aria-hidden="true">
        <svg
          viewBox="0 0 24 24"
          width="18"
          height="18"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
          <line x1="12" y1="9" x2="12" y2="13" />
          <line x1="12" y1="17" x2="12.01" y2="17" />
        </svg>
      </span>

      <div class="scan-warnings__title">
        <strong>
          {{ result.errors.length }}
          {{ result.errors.length === 1 ? 'path' : 'paths' }} could not be fully scanned.
        </strong>
        <span>Some entries were skipped while scanning.</span>
      </div>
    </div>

    <ul class="scan-warnings__list">
      <li
        v-for="item in result.errors"
        :key="`${item.path}:${item.message}`"
        class="scan-warnings__item"
      >
        <code>{{ item.path }}</code>
        <span>{{ item.message }}</span>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.scan-warnings {
  padding: 16px 20px;
  border: 1px solid #fde68a;
  border-radius: 12px;
  background: #fffbeb;
  color: #78350f;
}

.scan-warnings__header {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.scan-warnings__icon {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: #fef3c7;
  color: #b45309;
}

.scan-warnings__title {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.scan-warnings__title strong {
  font-size: 13px;
}

.scan-warnings__title span {
  color: #92400e;
  font-size: 12px;
}

.scan-warnings__list {
  display: grid;
  gap: 6px;
  margin: 12px 0 0;
  padding: 10px 0 0;
  border-top: 1px solid #fde68a;
  list-style: none;
}

.scan-warnings__item {
  display: grid;
  gap: 2px;
  font-size: 12px;
}

.scan-warnings__item code {
  overflow: hidden;
  color: #78350f;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.scan-warnings__item span {
  color: #92400e;
  word-break: break-word;
}
</style>