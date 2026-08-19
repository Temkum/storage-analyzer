import { ref } from 'vue'

import { scanDirectory } from '@/services/analyzer'
import type { ScanResult } from '@/types/scan'

export function useScanner() {
  const result = ref<ScanResult | null>(null)
  const isScanning = ref(false)
  const error = ref<string | null>(null)

  async function scan(path: string) {
    isScanning.value = true
    error.value = null

    try {
      result.value = await scanDirectory(path)
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
    } finally {
      isScanning.value = false
    }
  }

  return {
    result,
    isScanning,
    error,
    scan,
  }
}
