import { invoke } from '@tauri-apps/api/core'

import type { ScanResult } from '@/types/scan'

export async function scanDirectory(path: string): Promise<ScanResult> {
  return invoke<ScanResult>('scan_directory', { path })
}
