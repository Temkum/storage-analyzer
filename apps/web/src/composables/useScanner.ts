import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { ref } from 'vue'

import { scanDirectory } from '@/services/analyzer'
import type { ScanResult } from '@/types/scan'

export function useScanner() {
  const result = ref<ScanResult | null>(null)
  const isScanning = ref(false)
  const progress = ref(0)
  const scannedEntries = ref(0)
  const error = ref<string | null>(null)

  let unlistenProgress: UnlistenFn | null = null

  async function scan(path: string) {
    isScanning.value = true
    progress.value = 0
    scannedEntries.value = 0
    error.value = null

    unlistenProgress?.()

    unlistenProgress = await listen<number>('scan-progress', (event) => {
      scannedEntries.value = event.payload
    })

    try {
      result.value = await scanDirectory(path)
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
    } finally {
      isScanning.value = false
      unlistenProgress?.()
      unlistenProgress = null
    }
  }

  return {
    result,
    isScanning,
    progress,
    scannedEntries,
    error,
    scan,
  }
}
