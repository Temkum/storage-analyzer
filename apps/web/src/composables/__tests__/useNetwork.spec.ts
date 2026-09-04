import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import {
  getApplicationHistory,
  getNetworkHistory,
  getNetworkLive,
  getTopApplications,
} from '@/services/network'
import { LIVE_POLL_MS, useNetwork } from '@/composables/useNetwork'
import type { NetworkLive } from '@/types/network'

vi.mock('@/services/network', () => ({
  getNetworkLive: vi.fn<typeof getNetworkLive>(),
  getNetworkHistory: vi.fn<typeof getNetworkHistory>(),
  getApplicationHistory: vi.fn<typeof getApplicationHistory>(),
  getTopApplications: vi.fn<typeof getTopApplications>(),
}))

const mockedLive = vi.mocked(getNetworkLive)
const mockedHistory = vi.mocked(getNetworkHistory)
const mockedApps = vi.mocked(getApplicationHistory)
const mockedTop = vi.mocked(getTopApplications)

function liveSnapshot(overrides: Partial<NetworkLive> = {}): NetworkLive {
  return {
    updatedAt: 100,
    totals: [{ timestamp: 100, bytesReceived: 1200, bytesSent: 300 }],
    interfaces: [],
    applications: [],
    error: null,
    ...overrides,
  }
}

beforeEach(() => {
  vi.useFakeTimers()
  mockedLive.mockReset()
  mockedHistory.mockReset()
  mockedApps.mockReset()
  mockedTop.mockReset()
  mockedHistory.mockResolvedValue({
    since: 0,
    until: 1,
    bucketSeconds: 60,
    totals: [],
    interfaces: [],
  })
  mockedApps.mockResolvedValue([])
  mockedTop.mockResolvedValue([])
})

afterEach(() => {
  vi.useRealTimers()
})

describe('useNetwork', () => {
  it('polls live telemetry on the configured cadence', async () => {
    mockedLive.mockResolvedValue(liveSnapshot())

    const { live, isConnected } = useNetwork()

    expect(mockedLive).toHaveBeenCalledTimes(1)
    await vi.advanceTimersByTimeAsync(0)

    expect(live.value?.totals[0]?.bytesReceived).toBe(1200)
    expect(isConnected.value).toBe(true)

    await vi.advanceTimersByTimeAsync(LIVE_POLL_MS)
    await vi.advanceTimersByTimeAsync(LIVE_POLL_MS)

    expect(mockedLive).toHaveBeenCalledTimes(3)
  })

  it('stop() halts polling while the sampler keeps running in Rust', async () => {
    mockedLive.mockResolvedValue(liveSnapshot())

    const { stop } = useNetwork()
    await vi.advanceTimersByTimeAsync(0)

    stop()
    await vi.advanceTimersByTimeAsync(LIVE_POLL_MS * 5)

    expect(mockedLive).toHaveBeenCalledTimes(1)
  })

  it('surfaces an invocation failure without throwing', async () => {
    mockedLive.mockRejectedValue('sidecar unavailable')

    const { liveError, isConnected, pollLive } = useNetwork()
    await vi.advanceTimersByTimeAsync(0)

    expect(liveError.value).toBe('sidecar unavailable')
    expect(isConnected.value).toBe(false)

    // Recovery on a later tick.
    mockedLive.mockResolvedValue(liveSnapshot())
    await pollLive()

    expect(isConnected.value).toBe(true)
    expect(liveError.value).toBeNull()
  })

  it('loads history, applications and top apps for the active range', async () => {
    mockedLive.mockResolvedValue(liveSnapshot())
    mockedApps.mockResolvedValue([
      {
        appId: '/usr/bin/app',
        processName: 'app',
        executablePath: null,
        bytesReceived: 10,
        bytesSent: 5,
      },
    ])
    mockedTop.mockResolvedValue([])

    const { applications, setRange, refreshHistory } = useNetwork()
    await vi.advanceTimersByTimeAsync(0)

    expect(applications.value).toHaveLength(1)
    expect(mockedHistory).toHaveBeenCalledWith('24h')

    setRange('1h')
    await vi.advanceTimersByTimeAsync(0)

    expect(mockedHistory).toHaveBeenLastCalledWith('1h')
    expect(mockedTop).toHaveBeenLastCalledWith('1h')

    // A manual refresh keeps the current range.
    await refreshHistory()
    expect(mockedHistory).toHaveBeenLastCalledWith('1h')
  })

  it('reports history load errors', async () => {
    mockedLive.mockResolvedValue(liveSnapshot())
    mockedHistory.mockRejectedValue('database locked')

    const { historyError } = useNetwork()
    await vi.advanceTimersByTimeAsync(0)

    expect(historyError.value).toBe('database locked')
  })
})
