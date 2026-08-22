<script setup lang="ts">
import { basename } from '@/utils/paths'

const props = defineProps<{
  chain: string[]
  scanning: boolean
}>()

const emit = defineEmits<{
  navigate: [path: string]
}>()

function crumbLabel(index: number): string {
  const path = props.chain[index]
  if (!path) {
    return ''
  }

  return path === '/' ? '/' : basename(path)
}

function handleGo(index: number) {
  if (props.scanning) {
    return
  }

  const target = props.chain[index]
  if (target) {
    emit('navigate', target)
  }
}
</script>

<template>
  <nav class="breadcrumb" aria-label="Directory navigation">
    <span class="breadcrumb__label">Location</span>

    <ol class="breadcrumb__list">
      <li v-for="(entry, index) in chain" :key="entry" class="breadcrumb__item">
        <button type="button" class="breadcrumb__crumb"
          :class="{ 'breadcrumb__crumb--current': index === chain.length - 1 }"
          :disabled="scanning || index === chain.length - 1" :title="entry" @click="handleGo(index)">
          <svg v-if="scanning && index === chain.length - 1" class="breadcrumb__spinner" viewBox="0 0 24 24" width="14"
            height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <path d="M21 12a9 9 0 1 1-6.219-8.56" />
          </svg>

          <span>{{ crumbLabel(index) }}</span>
        </button>

        <span v-if="index < chain.length - 1" class="breadcrumb__sep" aria-hidden="true">/</span>
      </li>
    </ol>
  </nav>
</template>

<style scoped>
.breadcrumb {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
  padding: 12px 16px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #ffffff;
}

.breadcrumb__label {
  flex-shrink: 0;
  color: #94a3b8;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.breadcrumb__list {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 6px;
  margin: 0;
  padding: 0;
  overflow-x: auto;
  list-style: none;
}

.breadcrumb__item {
  display: inline-flex;
  min-width: 0;
  align-items: center;
  gap: 6px;
}

.breadcrumb__crumb {
  display: inline-flex;
  max-width: 220px;
  align-items: center;
  gap: 6px;
  padding: 5px 11px;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  background: #f8fafc;
  color: #475569;
  font: inherit;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition:
    border-color 150ms ease,
    background 150ms ease,
    color 150ms ease;
}

.breadcrumb__crumb span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.breadcrumb__crumb:hover:not(:disabled) {
  border-color: #cbd5e1;
  background: #f1f5f9;
  color: #0f172a;
}

.breadcrumb__crumb:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}

.breadcrumb__crumb--current {
  border-color: #0f172a;
  background: #0f172a;
  color: #ffffff;
  cursor: default;
}

.breadcrumb__crumb:disabled {
  cursor: default;
  opacity: 1;
}

.breadcrumb__sep {
  color: #cbd5e1;
  font-size: 13px;
}

.breadcrumb__spinner {
  flex-shrink: 0;
  animation: breadcrumb-spin 800ms linear infinite;
}

@keyframes breadcrumb-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 500px) {
  .breadcrumb__label {
    display: none;
  }
}
</style>
