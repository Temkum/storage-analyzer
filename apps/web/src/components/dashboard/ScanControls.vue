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

    <button type="submit" class="scan-controls__scan" :disabled="scanning || !path.trim()">
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
