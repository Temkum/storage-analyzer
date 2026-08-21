<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'

defineProps<{
  path: string
  scanning: boolean
  scannedEntries: number
}>()

const emit = defineEmits<{
  'update:path': [value: string]
  scan: []
}>()

async function chooseDirectory() {
  const selected = await open({
    directory: true,
    multiple: false,
  })

  if (typeof selected === 'string') {
    emit('update:path', selected)
  }
}
</script>

<template>
  <form class="scan-controls" @submit.prevent="emit('scan')">
    <div class="scan-controls__field">
      <label for="directory">Directory</label>

      <div class="scan-controls__input">
        <input id="directory" :value="path" type="text" placeholder="/home/user/Documents" :disabled="scanning" @input="
          emit(
            'update:path',
            ($event.target as HTMLInputElement).value,
          )
          " />

        <button type="button" class="scan-controls__browse" :disabled="scanning" @click="chooseDirectory">
          Browse
        </button>
      </div>
    </div>

    <button
      type="submit"
      class="scan-controls__scan"
      :disabled="scanning || !path.trim()"
    >
      <svg
        v-if="!scanning"
        class="scan-controls__scan-icon"
        viewBox="0 0 24 24"
        width="16"
        height="16"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>

      <svg
        v-else
        class="scan-controls__spinner"
        viewBox="0 0 24 24"
        width="16"
        height="16"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        aria-hidden="true"
      >
        <path d="M21 12a9 9 0 1 1-6.219-8.56" />
      </svg>

      {{ scanning ? 'Scanning...' : 'Scan Directory' }}
    </button>
  </form>

  <div v-if="scanning" class="scan-progress" role="status" aria-live="polite">
    <span class="scan-progress__indicator" />
    <span>
      Scanning {{ scannedEntries.toLocaleString() }} entries...
    </span>
  </div>
</template>

<style scoped>
.scan-controls__input {
  display: flex;
  gap: 8px;
}

.scan-controls__input input {
  flex: 1;
}

.scan-controls__browse {
  padding: 11px 16px;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  background: #f8fafc;
  color: #0f172a;
  font: inherit;
  font-weight: 600;
  cursor: pointer;
}

.scan-controls__browse:hover:not(:disabled) {
  background: #f1f5f9;
}

.scan-controls__scan {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px 24px;
  border: 1px solid transparent;
  border-radius: 8px;
  background: #0f172a;
  color: #fff;
  font: inherit;
  font-weight: 600;
  cursor: pointer;
  transition:
    background 150ms ease,
    transform 150ms ease,
    box-shadow 150ms ease;
}

.scan-controls__scan:hover:not(:disabled) {
  background: #1e293b;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(15, 23, 42, 0.18);
}

.scan-controls__scan:active:not(:disabled) {
  transform: translateY(0);
  box-shadow: none;
}

.scan-controls__scan:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}

.scan-controls__scan:disabled {
  background: #cbd5e1;
  color: #f8fafc;
  cursor: not-allowed;
}

.scan-controls__scan-icon {
  flex-shrink: 0;
}

.scan-controls__spinner {
  flex-shrink: 0;
  animation: scan-spin 800ms linear infinite;
}

@keyframes scan-spin {
  to {
    transform: rotate(360deg);
  }
}

.scan-progress {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border: 1px solid #dbeafe;
  border-radius: 10px;
  background: #eff6ff;
  color: #1e40af;
  font-size: 13px;
  font-weight: 600;
}

.scan-progress__indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #2563eb;
  animation: scan-pulse 1s ease-in-out infinite;
}

@keyframes scan-pulse {

  0%,
  100% {
    opacity: 0.35;
  }

  50% {
    opacity: 1;
  }
}
</style>
