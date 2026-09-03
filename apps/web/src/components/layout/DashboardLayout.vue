<script setup lang="ts">
import { computed } from 'vue'

import { useScrollSpy } from '@/composables/useScrollSpy'

const props = defineProps<{
  status: 'idle' | 'scanning' | 'error'
}>()

const NAV_ITEMS = [
  {
    id: 'overview',
    label: 'Overview',
    svgPath: 'M3 13h8V3H3v10zm0 8h8v-6H3v6zm10 0h8v-10h-8v10zm0-18v6h8V3h-8z'
  },
  {
    id: 'storage',
    label: 'Storage',
    svgPath: 'M20 13H4c-.55 0-1-.45-1-1V6c0-.55.45-1 1-1h16c.55 0 1 .45 1 1v6c0 .55-.45 1-1 1zM4 19h16c.55 0 1-.45 1-1v-4c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1v4c0 .55.45 1 1 1z'
  },
  {
    id: 'network',
    label: 'Network',
    svgPath: 'M12 11c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm6 2c0-3.31-2.69-6-6-6s-6 2.69-6 6c0 2.22 1.21 4.15 3 5.19l1-1.73c-1.19-.7-2-1.97-2-3.46 0-2.21 1.79-4 4-4s4 1.79 4 4c0 1.49-.81 2.76-2 3.46l1 1.73c1.79-1.04 3-2.97 3-5.19z'
  },
  {
    id: 'volumes',
    label: 'Volumes',
    svgPath: 'M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-2 10h-4v4h-2v-4H7v-2h4V7h2v4h4v2z'
  },
] as const

const { activeId } = useScrollSpy(NAV_ITEMS.map((item) => item.id))

const activeItem = computed(() => {
  return NAV_ITEMS.find((item) => item.id === activeId.value) ?? NAV_ITEMS[0]
})

const statusLabel = computed(() => {
  if (props.status === 'scanning') return 'Scanning...'
  if (props.status === 'error') return 'Error'
  return 'Ready'
})

function jumpTo(id: string) {
  document.getElementById(id)?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand">
        <div class="brand__text">
          <strong>System Analyzer</strong>
          <span>Disk and Network Utility</span>
        </div>
      </div>

      <nav class="sidebar__nav" aria-label="Dashboard sections">
        <a v-for="item in NAV_ITEMS" :key="item.id" class="sidebar__link"
          :class="{ 'sidebar__link--active': activeId === item.id }" :href="`#${item.id}`" :title="item.label"
          :aria-current="activeId === item.id ? 'location' : undefined" @click.prevent="jumpTo(item.id)">
          <svg class="sidebar__icon" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <path :d="item.svgPath" />
          </svg>
          <span class="sidebar__label">{{ item.label }}</span>
        </a>
      </nav>

      <div class="sidebar__footer">
        <div class="status-badge" :class="`status-badge--${status}`" role="status">
          <span v-if="status === 'scanning'" class="status-badge__dot" aria-hidden="true" />
          {{ statusLabel }}
        </div>
      </div>
    </aside>

    <main class="main">
      <header class="topbar">
        <div>
          <span class="topbar__section">Dashboard / {{ activeItem.label }}</span>
          <h1>{{ activeItem.label }} Details</h1>
        </div>

        <div class="topbar__status">
          <div class="status-badge" :class="`status-badge--${status}`" role="status">
            <span v-if="status === 'scanning'" class="status-badge__dot" aria-hidden="true" />
            {{ statusLabel }}
          </div>
        </div>
      </header>

      <div class="content">
        <slot />
      </div>
    </main>
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  min-height: 100vh;
  background: #f8fafc;
  color: #0f172a;
}

.sidebar {
  position: fixed;
  top: 0;
  bottom: 0;
  left: 0;
  z-index: 20;
  display: flex;
  width: 240px;
  flex-direction: column;
  padding: 20px 16px;
  border-right: 1px solid #e2e8f0;
  background: #ffffff;
}

.brand {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 8px 24px;
}

.brand__mark {
  display: grid;
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  place-items: center;
  border-radius: 8px;
  background: #0f172a;
  color: #ffffff;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: -0.03em;
}

.brand__text strong,
.brand__text span {
  display: block;
  line-height: 1.25;
}

.brand__text strong {
  font-size: 13px;
  font-weight: 600;
}

.brand__text span {
  margin-top: 2px;
  color: #64748b;
  font-size: 11px;
}

.sidebar__nav {
  display: grid;
  gap: 2px;
}

.sidebar__link {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  border-radius: 6px;
  color: #64748b;
  font-size: 13px;
  font-weight: 500;
  text-decoration: none;
  transition: background-color 150ms ease, color 150ms ease;
}

.sidebar__link:hover {
  background: #f1f5f9;
  color: #0f172a;
}

.sidebar__link--active {
  background: #eff6ff;
  color: #2563eb;
  font-weight: 600;
}

.sidebar__link:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: -1px;
}

.sidebar__icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
}

.sidebar__footer {
  display: flex;
  align-items: center;
  margin-top: auto;
  padding-top: 16px;
  border-top: 1px solid #f1f5f9;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 600;
  line-height: 1;
}

.status-badge__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  animation: pulse 1.5s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

.status-badge--idle {
  background: #f0fdf4;
  color: #166534;
}

.status-badge--scanning {
  background: #fff7ed;
  color: #c2410c;
}

.status-badge--error {
  background: #fef2f2;
  color: #dc2626;
}

.main {
  flex: 1;
  min-width: 0;
  margin-left: 240px;
}

.topbar {
  position: sticky;
  top: 0;
  z-index: 10;
  display: flex;
  min-height: 68px;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  padding: 12px 32px;
  border-bottom: 1px solid #e2e8f0;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(8px);
}

.topbar__section {
  color: #64748b;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.03em;
  text-transform: uppercase;
}

.topbar h1 {
  margin: 2px 0 0;
  font-size: 18px;
  font-weight: 600;
  line-height: 1.2;
}

.topbar__status {
  display: flex;
  align-items: center;
}

.content {
  display: grid;
  width: min(1360px, 100%);
  gap: 24px;
  margin: 0 auto;
  padding: 24px 32px 48px;
}

@keyframes pulse {

  0%,
  100% {
    opacity: 1;
  }

  50% {
    opacity: 0.3;
  }
}

@media (max-width: 900px) {
  .sidebar {
    width: 64px;
    padding: 16px 8px;
  }

  .brand__text,
  .sidebar__label,
  .sidebar__footer,
  .topbar__status {
    display: none;
  }

  .brand {
    justify-content: center;
    padding: 4px 0 16px;
  }

  .sidebar__link {
    justify-content: center;
    padding: 10px;
  }

  .main {
    margin-left: 64px;
  }

  .topbar {
    padding: 12px 20px;
  }

  .content {
    padding: 20px;
  }
}

@media (max-width: 600px) {
  .topbar h1 {
    font-size: 16px;
  }

  .content {
    padding: 16px;
  }
}
</style>
