import { invoke } from '@tauri-apps/api/core'

import type { ScanResult } from '@/types/scan'

export async function scanDirectory(path: string): Promise<ScanResult> {
  const raw = await invoke<string>('scan_directory', { path })
  return JSON.parse(raw) as ScanResult
}

export async function cancelScan(): Promise<void> {
  await invoke('cancel_scan')
}

export async function revealInFileManager(path: string): Promise<void> {
  await invoke('reveal_in_file_manager', { path })
}
