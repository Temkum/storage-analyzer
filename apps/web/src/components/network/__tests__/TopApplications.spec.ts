import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import TopApplications from '@/components/network/TopApplications.vue'
import type { ApplicationUsage } from '@/types/network'

const apps: ApplicationUsage[] = [
  {
    appId: '/usr/bin/chrome',
    processName: 'chrome',
    executablePath: '/usr/bin/chrome',
    bytesReceived: 1.2 * 1024 ** 3,
    bytesSent: 180 * 1024 ** 2,
  },
  {
    appId: '/usr/bin/firefox',
    processName: 'firefox',
    executablePath: '/usr/bin/firefox',
    bytesReceived: 640 * 1024 ** 2,
    bytesSent: 92 * 1024 ** 2,
  },
]

describe('TopApplications', () => {
  it('renders the ranked application list with RX/TX totals', () => {
    const wrapper = mount(TopApplications, {
      props: { applications: apps, loading: false },
    })

    const itemTexts = wrapper.findAll('.apps__item').map((item) => item.text())

    expect(itemTexts).toHaveLength(2)
    expect(itemTexts[0]).toContain('chrome')
    expect(itemTexts[0]).toContain('GB RX')
    expect(itemTexts[0]).toContain('MB TX')
    expect(itemTexts[0]).toMatch(/^1chrome/)
    expect(itemTexts[1]).toMatch(/^2firefox/)
  })

  it('shows a loading state', () => {
    const wrapper = mount(TopApplications, {
      props: { applications: [], loading: true },
    })

    expect(wrapper.find('[role="status"]').text()).toContain('Loading')
  })

  it('shows an empty state and explains zero attribution is not failure', () => {
    const wrapper = mount(TopApplications, {
      props: { applications: [], loading: false },
    })

    expect(wrapper.text()).toMatch(/No attributable application traffic/i)
  })
})
