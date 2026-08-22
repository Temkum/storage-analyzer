import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { ref } from 'vue'

import { cancelScan, scanDirectory } from '@/services/analyzer'
import type { ScanResult } from '@/types/scan'

const SCAN_CANCELLED_MESSAGE = 'SCAN_CANCELLED'

export function useScanner() {
  const result = ref<ScanResult | null>(null)
  const isScanning = ref(false)
  const scannedEntries = ref(0)
  const error = ref<string | null>(null)
  const cancelled = ref(false)

  let unlistenProgress: UnlistenFn | null = null

  function clearError() {
    error.value = null
  }

  async function scan(path: string) {
    if (isScanning.value) {
      return
    }

    isScanning.value = true
    scannedEntries.value = 0
    error.value = null
    cancelled.value = false

    unlistenProgress?.()

    unlistenProgress = await listen<number>('scan-progress', (event) => {
      scannedEntries.value = event.payload
    })

    try {
      result.value = await scanDirectory(path)
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)

      if (message === SCAN_CANCELLED_MESSAGE) {
        cancelled.value = true
      } else {
        error.value = message
      }
    } finally {
      isScanning.value = false
      unlistenProgress?.()
      unlistenProgress = null
    }
  }

  async function cancel() {
    try {
      await cancelScan()
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      error.value = message
    }
  }

  return {
    result,
    isScanning,
    scannedEntries,
    cancelled,
    error,
    clearError,
    scan,
    cancel,
  }
}
