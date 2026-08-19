<script setup lang="ts">
import { ref } from 'vue'

import LargestFiles from '@/components/dashboard/LargestFiles.vue'
import ScanSummary from '@/components/dashboard/ScanSummary.vue'
import { useScanner } from '@/composables/useScanner'
import DirectoryBreakdown from '@/components/dashboard/DirectoryBreakdown.vue'

const path = ref('/tmp/system-analyzer-test')

const {
  result,
  isScanning,
  error,
  scan,
} = useScanner()

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

    <ScanSummary
      v-if="result"
      :result="result"
    />

    <LargestFiles
      v-if="result"
      :result="result"
    />

    <DirectoryBreakdown
      v-if="result"
      :result="result"
    />
  </main>
</template>
