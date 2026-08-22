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
      <label class="scan-controls__label" for="directory">Directory</label>

      <div class="scan-controls__input">
        <div class="scan-controls__field-wrap">
          <svg class="scan-controls__icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor"
            stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
          </svg>

          <input id="directory" class="scan-controls__input-field" :value="path" type="text"
            placeholder="/home/user/Documents" :disabled="scanning" @input="
              emit(
                'update:path',
                ($event.target as HTMLInputElement).value,
              )
              " />
        </div>

        <button type="button" class="scan-controls__browse" :disabled="scanning" @click="chooseDirectory">
          Browse
        </button>
      </div>
    </div>

    <button type="submit" class="scan-controls__scan" :disabled="scanning || !path.trim()">
      <svg v-if="!scanning" class="scan-controls__scan-icon" viewBox="0 0 24 24" width="16" height="16" fill="none"
        stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>

      <svg v-else class="scan-controls__spinner" viewBox="0 0 24 24" width="16" height="16" fill="none"
        stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
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
.scan-controls {
  display: flex;
  align-items: flex-end;
  gap: 16px;
  padding: 22px 24px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #ffffff;
}

.scan-controls__field {
  min-width: 0;
  flex: 1;
}

.scan-controls__label {
  display: block;
  margin: 0 0 8px;
  color: #64748b;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.scan-controls__input {
  display: flex;
  min-width: 0;
  align-items: stretch;
  gap: 8px;
}

.scan-controls__field-wrap {
  display: flex;
  min-width: 0;
  flex: 1;
  align-items: center;
  gap: 10px;
  padding: 0 4px 0 12px;
  border: 1px solid #cbd5e1;
  border-radius: 10px;
  background: #f8fafc;
  transition:
    border-color 150ms ease,
    box-shadow 150ms ease,
    background-color 150ms ease;
}

.scan-controls__field-wrap:focus-within {
  border-color: #2563eb;
  background: #ffffff;
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.12);
}

.scan-controls__icon {
  flex-shrink: 0;
  color: #94a3b8;
}

.scan-controls__input-field {
  min-width: 0;
  flex: 1;
  align-self: stretch;
  padding: 11px 12px 11px 0;
  border: none;
  background: transparent;
  color: #0f172a;
  font: inherit;
}

.scan-controls__input-field:focus {
  outline: none;
}

.scan-controls__input-field:disabled {
  color: #94a3b8;
  cursor: not-allowed;
}

.scan-controls__input-field::placeholder {
  color: #94a3b8;
}

.scan-controls__browse {
  flex-shrink: 0;
  padding: 11px 16px;
  border: 1px solid #cbd5e1;
  border-radius: 10px;
  background: #ffffff;
  color: #334155;
  font: inherit;
  font-weight: 600;
  cursor: pointer;
  transition:
    border-color 150ms ease,
    background 150ms ease;
}

.scan-controls__browse:hover:not(:disabled) {
  border-color: #94a3b8;
  background: #f1f5f9;
}

.scan-controls__browse:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}

.scan-controls__browse:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.scan-controls__scan {
  display: inline-flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px 28px;
  border: 1px solid transparent;
  border-radius: 10px;
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
  margin-top: 12px;
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

@media (max-width: 700px) {
  .scan-controls {
    align-items: stretch;
    flex-direction: column;
  }

  .scan-controls__scan {
    width: 100%;
  }
}
</style>
