import { beforeEach, describe, expect, it, vi } from 'vitest'

import { invoke } from '@tauri-apps/api/core'

import {
  getApplicationHistory,
  getNetworkHistory,
  getNetworkLive,
  getTopApplications,
} from '@/services/network'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn<() => Promise<unknown>>(),
}))

const mockedInvoke = vi.mocked(invoke)

const NOW = 1_700_000_000

beforeEach(() => {
  mockedInvoke.mockReset()
})

describe('network service', () => {
  it('live command targets get_network_live with no arguments', async () => {
    mockedInvoke.mockResolvedValue({
      updatedAt: null,
      totals: [],
      interfaces: [],
      applications: [],
      error: null,
    })

    await getNetworkLive()

    expect(mockedInvoke).toHaveBeenCalledWith('get_network_live')
  })

  it('history command maps a 1h range to [now-3600, now)', async () => {
    mockedInvoke.mockResolvedValue({
      since: 0,
      until: 0,
      bucketSeconds: 60,
      totals: [],
      interfaces: [],
    })

    await getNetworkHistory('1h', NOW)

    expect(mockedInvoke).toHaveBeenCalledWith('get_network_history', {
      since: NOW - 60 * 60,
      until: NOW,
    })
  })

  it('history command maps a 24h range to [now-86400, now)', async () => {
    mockedInvoke.mockResolvedValue({
      since: 0,
      until: 0,
      bucketSeconds: 900,
      totals: [],
      interfaces: [],
    })

    await getNetworkHistory('24h', NOW)

    expect(mockedInvoke).toHaveBeenCalledWith('get_network_history', {
      since: NOW - 24 * 60 * 60,
      until: NOW,
    })
  })

  it('application history uses the same window math', async () => {
    mockedInvoke.mockResolvedValue([])

    await getApplicationHistory('6h', NOW)

    expect(mockedInvoke).toHaveBeenCalledWith('get_application_history', {
      since: NOW - 6 * 60 * 60,
      until: NOW,
    })
  })

  it('top applications forwards the limit', async () => {
    mockedInvoke.mockResolvedValue([])

    await getTopApplications('24h', 5, NOW)

    expect(mockedInvoke).toHaveBeenCalledWith('get_top_applications', {
      since: NOW - 24 * 60 * 60,
      until: NOW,
      limit: 5,
    })
  })
})
