import { invoke } from '@tauri-apps/api/core'

import {
  HISTORY_RANGE_SECONDS,
  type ApplicationUsage,
  type HistoryRange,
  type NetworkHistory,
  type NetworkLive,
} from '@/types/network'

/** Live telemetry from the always-running sampler's ring buffers. */
export async function getNetworkLive(): Promise<NetworkLive> {
  return invoke<NetworkLive>('get_network_live')
}

/** Computes the [since, until) window for a named range. */
function rangeWindow(range: HistoryRange, nowSeconds?: number): { since: number; until: number } {
  const now = nowSeconds ?? Math.floor(Date.now() / 1000)
  return { since: now - HISTORY_RANGE_SECONDS[range], until: now }
}

/** Aggregated interface history over [now - range, now). */
export async function getNetworkHistory(
  range: HistoryRange,
  nowSeconds?: number,
): Promise<NetworkHistory> {
  const { since, until } = rangeWindow(range, nowSeconds)
  return invoke<NetworkHistory>('get_network_history', { since, until })
}

/** Per-application usage totals over [now - range, now). */
export async function getApplicationHistory(
  range: HistoryRange,
  nowSeconds?: number,
): Promise<ApplicationUsage[]> {
  const { since, until } = rangeWindow(range, nowSeconds)
  return invoke<ApplicationUsage[]>('get_application_history', { since, until })
}

/** Top-N application ranking over [now - range, now). */
export async function getTopApplications(
  range: HistoryRange,
  limit = 10,
  nowSeconds?: number,
): Promise<ApplicationUsage[]> {
  const { since, until } = rangeWindow(range, nowSeconds)
  return invoke<ApplicationUsage[]>('get_top_applications', { since, until, limit })
}
