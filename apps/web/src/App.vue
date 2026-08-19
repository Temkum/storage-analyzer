<script setup lang="ts">
import { ref } from 'vue'

import { useScanner } from '@/composables/useScanner'

const path = ref('/tmp/system-analyzer-test')

const { result, isScanning, error, scan } = useScanner()

async function handleScan() {
  await scan(path.value)
}
</script>

<template>
  <main>
    <h1>System Analyzer</h1>

    <form @submit.prevent="handleScan">
      <label for="directory">
        Directory
      </label>

      <input
        id="directory"
        v-model="path"
        type="text"
        placeholder="/path/to/directory"
        :disabled="isScanning"
      />

      <button
        type="submit"
        :disabled="isScanning || !path.trim()"
      >
        {{ isScanning ? 'Scanning...' : 'Scan Directory' }}
      </button>
    </form>

    <p v-if="error">
      {{ error }}
    </p>

    <pre v-if="result">{{ JSON.stringify(result, null, 2) }}</pre>
  </main>
</template>
