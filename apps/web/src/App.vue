<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const path = ref('/tmp/system-analyzer-test')
const result = ref('')
const error = ref('')

async function scan() {
  result.value = ''
  error.value = ''

  try {
    result.value = await invoke<string>('scan_directory', {
      path: path.value,
    })
  } catch (err) {
    error.value = String(err)
  }
}
</script>

<template>
  <main>
    <h1>System Analyzer</h1>

    <input v-model="path" />

    <button @click="scan">
      Scan Directory
    </button>

    <pre v-if="result">{{ result }}</pre>

    <p v-if="error">
      {{ error }}
    </p>
  </main>
</template>
