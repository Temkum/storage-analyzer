import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import NetworkOverview from '@/components/network/NetworkOverview.vue'

describe('NetworkOverview', () => {
  it('renders the connected rates', () => {
    const wrapper = mount(NetworkOverview, {
      props: {
        bytesReceivedPerSecond: 12.4 * 1024 * 1024,
        bytesSentPerSecond: 2.1 * 1024 * 1024,
        connected: true,
      },
    })

    expect(wrapper.find('.overview__value').text()).toContain('MB/s')
    expect(wrapper.text()).toContain('Received')
    expect(wrapper.text()).toContain('Sent')
  })

  it('shows a placeholder when disconnected', () => {
    const wrapper = mount(NetworkOverview, {
      props: { bytesReceivedPerSecond: 0, bytesSentPerSecond: 0, connected: false },
    })

    expect(wrapper.text()).toContain('—')
  })

  it('distinguishes total interface traffic from attributable traffic', () => {
    const wrapper = mount(NetworkOverview, {
      props: { bytesReceivedPerSecond: 1, bytesSentPerSecond: 1, connected: true },
    })

    expect(wrapper.text()).toMatch(/Received.*are total interface traffic/i)
    expect(wrapper.text()).toMatch(/attributable through \/proc/i)
  })
})
