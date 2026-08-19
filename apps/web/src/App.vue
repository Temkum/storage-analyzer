<script setup lang="ts">
import { ref } from 'vue'

import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import DirectoryBreakdown from '@/components/dashboard/DirectoryBreakdown.vue'
import FileTypeBreakdown from '@/components/dashboard/FileTypeBreakdown.vue'
import LargestFiles from '@/components/dashboard/LargestFiles.vue'
import ScanControls from '@/components/dashboard/ScanControls.vue'
import ScanSummary from '@/components/dashboard/ScanSummary.vue'
import { useScanner } from '@/composables/useScanner'
import StorageUsage from '@/components/dashboard/StorageUsage.vue'
import ScanWarnings from '@/components/dashboard/ScanWarnings.vue'

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
  <DashboardLayout :scanning="isScanning">
    <ScanControls v-model:path="path" :scanning="isScanning" @scan="handleScan" />

    <p v-if="error">
      {{ error }}
    </p>

    <template v-if="result">
      <ScanWarnings :result="result" />

      <ScanSummary :result="result" />

      <StorageUsage :result="result" />

      <LargestFiles :result="result" />

      <DirectoryBreakdown :result="result" />

      <FileTypeBreakdown :result="result" />
    </template>
  </DashboardLayout>
</template>
