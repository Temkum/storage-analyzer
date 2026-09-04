import { computed, getCurrentScope, onUnmounted, ref } from 'vue'

import {
  getApplicationHistory,
  getNetworkHistory,
  getNetworkLive,
  getTopApplications,
} from '@/services/network'
import type { ApplicationUsage, HistoryRange, NetworkHistory, NetworkLive } from '@/types/network'

/** Poll cadence for the live endpoint while a consumer is mounted. */
export const LIVE_POLL_MS = 1000

export const HISTORY_RANGES: HistoryRange[] = ['1h', '6h', '24h']

/**
 * Network Analyzer state.
 *
 * Lifecycle (Phase 0 decision): the Rust sampler runs for the lifetime of
 * the desktop app — this composable only starts/stops *consuming* it. Live
 * polling begins with `start()` (called on mount) and stops with `stop()`
 * / component unmount; the sampler keeps sampling either way.
 */
export function useNetwork(options: { immediate?: boolean } = {}) {
  const live = ref<NetworkLive | null>(null)
  const liveError = ref<string | null>(null)
  const isConnected = ref(false)

  const range = ref<HistoryRange>('24h')
  const history = ref<NetworkHistory | null>(null)
  const applications = ref<ApplicationUsage[]>([])
  const topApplications = ref<ApplicationUsage[]>([])
  const isHistoryLoading = ref(false)
  const historyError = ref<string | null>(null)

  let pollTimer: ReturnType<typeof setInterval> | null = null

  function errorMessage(err: unknown): string {
    return err instanceof Error ? err.message : String(err)
  }

  async function pollLive() {
    try {
      const snapshot = await getNetworkLive()
      live.value = snapshot
      isConnected.value = true

      // The backend keeps serving data even when the sidecar is restarting;
      // a monitor-level error is surfaced without breaking the page.
      liveError.value = snapshot.error
    } catch (err) {
      isConnected.value = false
      liveError.value = errorMessage(err)
    }
  }

  /** Starts live polling. Safe to call repeatedly; only the first wins. */
  function start() {
    if (pollTimer !== null) {
      return
    }

    void pollLive()
    pollTimer = setInterval(() => void pollLive(), LIVE_POLL_MS)
  }

  /** Stops live polling. The sampler keeps running in the background. */
  function stop() {
    if (pollTimer !== null) {
      clearInterval(pollTimer)
      pollTimer = null
    }
  }

  async function refreshHistory() {
    isHistoryLoading.value = true
    historyError.value = null

    try {
      const [historyResult, appsResult, topResult] = await Promise.all([
        getNetworkHistory(range.value),
        getApplicationHistory(range.value),
        getTopApplications(range.value),
      ])

      history.value = historyResult
      applications.value = appsResult
      topApplications.value = topResult
    } catch (err) {
      historyError.value = errorMessage(err)
    } finally {
      isHistoryLoading.value = false
    }
  }

  function setRange(next: HistoryRange) {
    if (range.value === next) {
      return
    }

    range.value = next
    void refreshHistory()
  }

  /** Latest total throughput (bytes/second) across all interfaces. */
  const currentThroughput = computed(() => {
    const totals = live.value?.totals ?? []
    const latest = totals[totals.length - 1]

    if (!latest) {
      return { bytesReceived: 0, bytesSent: 0 }
    }

    return { bytesReceived: latest.bytesReceived, bytesSent: latest.bytesSent }
  })

  if (options.immediate !== false) {
    start()
    void refreshHistory()

    if (getCurrentScope()) {
      onUnmounted(stop)
    }
  }

  return {
    live,
    liveError,
    isConnected,
    range,
    history,
    applications,
    topApplications,
    isHistoryLoading,
    historyError,
    currentThroughput,
    start,
    stop,
    refreshHistory,
    setRange,
    pollLive,
  }
}
