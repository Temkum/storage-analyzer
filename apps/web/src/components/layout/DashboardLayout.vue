<script setup lang="ts">
import { computed } from 'vue'

import { useScrollSpy } from '@/composables/useScrollSpy'

const props = defineProps<{
  status: 'idle' | 'scanning' | 'error'
}>()

const NAV_ITEMS = [
  { id: 'overview', label: 'Overview', icon: '◈' },
  { id: 'storage', label: 'Storage', icon: '▤' },
  { id: 'network', label: 'Network', icon: '≋' },
  { id: 'volumes', label: 'Volumes', icon: '◫' },
] as const

const { activeId } = useScrollSpy(NAV_ITEMS.map((item) => item.id))

const statusLabel = computed(() => {
  if (props.status === 'scanning') {
    return 'Scanning…'
  }

  if (props.status === 'error') {
    return 'Error'
  }

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
        <div>
          <strong>System Analyzer</strong>
          <span>Disk Analyzer</span>
        </div>
      </div>

      <nav class="sidebar__nav" aria-label="Dashboard sections">
        <a v-for="item in NAV_ITEMS" :key="item.id" class="sidebar__link"
          :class="{ 'sidebar__link--active': activeId === item.id }" :href="`#${item.id}`" :title="item.label"
          :aria-current="activeId === item.id ? 'location' : undefined" @click.prevent="jumpTo(item.id)">
          <span class="sidebar__icon" aria-hidden="true">{{ item.icon }}</span>
          <span class="sidebar__label">{{ item.label }}</span>
        </a>
      </nav>

      <div class="sidebar__footer">
        <span class="status-badge" :class="`status-badge--${status}`" role="status">
          {{ statusLabel }}
        </span>
      </div>
    </aside>

    <main class="main">
      <header class="topbar">
        <div>
          <span class="topbar__section">Storage</span>
          <h1>Disk Analyzer</h1>
        </div>

        <div class="topbar__status">
          <span class="status-badge" :class="`status-badge--${status}`" role="status">
            {{ statusLabel }}
          </span>
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
  inset: 0 auto 0 0;
  z-index: 20;
  display: flex;
  width: 240px;
  flex-direction: column;
  padding: 24px 16px;
  border-right: 1px solid #e2e8f0;
  background: #ffffff;
}

.brand {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 4px 8px 28px;
}

.brand__mark {
  display: grid;
  width: 34px;
  height: 34px;
  place-items: center;
  border-radius: 9px;
  background: #0f172a;
  color: #ffffff;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: -0.03em;
}

.brand strong,
.brand span {
  display: block;
}

.brand strong {
  font-size: 13px;
}

.brand span {
  margin-top: 2px;
  color: #64748b;
  font-size: 11px;
}

.sidebar__nav {
  display: grid;
  gap: 4px;
}

.sidebar__link {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 8px;
  color: #64748b;
  font-size: 13px;
  font-weight: 600;
  text-decoration: none;
}

.sidebar__link:hover {
  background: #f8fafc;
  color: #0f172a;
}

.sidebar__link--active {
  background: #f1f5f9;
  color: #0f172a;
}

.sidebar__link:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: -2px;
}

.sidebar__icon {
  width: 18px;
  flex-shrink: 0;
  text-align: center;
  font-size: 15px;
}

.sidebar__footer {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: auto;
  padding: 12px;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  padding: 5px 11px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
  line-height: 1;
}

.status-badge--idle {
  background: #dcfce7;
  color: #166534;
}

.status-badge--scanning {
  background: #ffedd5;
  color: #9a3412;
}

.status-badge--error {
  background: #fee2e2;
  color: #b91c1c;
}

.main {
  width: calc(100% - 240px);
  min-width: 0;
  margin-left: 240px;
}

.topbar {
  display: flex;
  min-height: 82px;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  padding: 18px 40px;
  border-bottom: 1px solid #e2e8f0;
  background: #ffffff;
}

.topbar__section {
  color: #64748b;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.topbar h1 {
  margin: 2px 0 0;
  font-size: 20px;
  line-height: 1.2;
}

.topbar__status {
  display: flex;
  align-items: center;
  gap: 8px;
}

.content {
  display: grid;
  width: min(1440px, 100%);
  gap: 24px;
  margin: 0 auto;
  padding: 32px 40px 48px;
  align-content: start;
}

@media (max-width: 900px) {
  .sidebar {
    position: static;
    width: 68px;
    padding: 18px 10px;
  }

  .brand>div:last-child,
  .sidebar__label,
  .sidebar__footer {
    display: none;
  }

  .brand {
    justify-content: center;
    padding: 4px 0 28px;
  }

  .sidebar__link {
    justify-content: center;
  }

  .sidebar__link--active {
    padding: 10px;
  }

  .main {
    width: calc(100% - 68px);
    margin-left: 0;
  }

  .topbar {
    padding: 18px 24px;
  }

  .content {
    padding: 24px;
  }
}

@media (max-width: 600px) {
  .topbar {
    align-items: flex-start;
    flex-direction: column;
  }

  .topbar__status {
    align-self: stretch;
    justify-content: flex-start;
  }

  .content {
    padding: 18px;
  }
}
</style>
