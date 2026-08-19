<script setup lang="ts">
defineProps<{
  path: string
  scanning: boolean
}>()

const emit = defineEmits<{
  'update:path': [value: string]
  scan: []
}>()
</script>

<template>
  <form
    class="scan-controls"
    @submit.prevent="emit('scan')"
  >
    <div class="scan-controls__field">
      <label for="directory">
        Directory
      </label>

      <input
        id="directory"
        :value="path"
        type="text"
        placeholder="/home/user/Documents"
        :disabled="scanning"
        @input="
          emit(
            'update:path',
            ($event.target as HTMLInputElement).value,
          )
        "
      />
    </div>

    <button
      type="submit"
      :disabled="scanning || !path.trim()"
    >
      {{ scanning ? 'Scanning...' : 'Scan Directory' }}
    </button>
  </form>
</template>

<style scoped>
.scan-controls {
  display: flex;
  align-items: flex-end;
  gap: 12px;
  padding: 20px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #fff;
}

.scan-controls__field {
  flex: 1;
}

.scan-controls label {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  font-weight: 600;
}

.scan-controls input {
  width: 100%;
  box-sizing: border-box;
  padding: 11px 12px;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  font: inherit;
}

.scan-controls button {
  padding: 11px 18px;
  border: 0;
  border-radius: 8px;
  background: #0f172a;
  color: #fff;
  font: inherit;
  font-weight: 600;
  cursor: pointer;
}

.scan-controls button:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

@media (max-width: 700px) {
  .scan-controls {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
