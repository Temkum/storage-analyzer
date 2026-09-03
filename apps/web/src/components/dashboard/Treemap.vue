<script setup lang="ts">
import { computed, ref } from 'vue'

import type { ScanResult } from '@/types/scan'
import { formatBytes } from '@/utils/format'
import { basename, isDirectChild, normalizePath } from '@/utils/paths'

interface TreemapItem {
  id: string
  label: string
  path: string
  size: number
  percentage: number
  isDir: boolean
}

interface Rect {
  item: TreemapItem
  x: number
  y: number
  width: number
  height: number
  colorIndex: number
}

const props = defineProps<{
  result: ScanResult
  navigating: boolean
}>()

const emit = defineEmits<{
  navigate: [path: string]
}>()

const VIEWBOX_WIDTH = 1000
const VIEWBOX_HEIGHT = 520
const GAP = 2

const hoveredItem = ref<TreemapItem | null>(null)

// Accessible, distinct palette scale (Dark slate -> Indigo -> Blue -> Cyan -> Teal -> Amber)
const TILE_PALETTE = [
  { fill: '#1e293b', border: '#334155', text: '#f8fafc', muted: '#94a3b8' },
  { fill: '#312e81', border: '#4338ca', text: '#e0e7ff', muted: '#a5b4fc' },
  { fill: '#1e40af', border: '#1d4ed8', text: '#dbeafe', muted: '#93c5fd' },
  { fill: '#0369a1', border: '#0284c7', text: '#e0f2fe', muted: '#7dd3fc' },
  { fill: '#0f766e', border: '#0d9488', text: '#ccfbf1', muted: '#5eead4' },
  { fill: '#b45309', border: '#d97706', text: '#fef3c7', muted: '#fcd34d' },
]

const rootPath = computed(() => normalizePath(props.result.rootPath))

const items = computed<TreemapItem[]>(() => {
  const root = rootPath.value

  const directories = props.result.directories
    .filter((directory) => isDirectChild(directory.path, root))
    .filter((directory) => directory.size > 0)

  const directorySize = directories.reduce(
    (total, directory) => total + directory.size,
    0,
  )

  const rootFilesSize = Math.max(props.result.totalSize - directorySize, 0)

  const values: TreemapItem[] = directories.map((directory) => ({
    id: directory.path,
    label: basename(directory.path),
    path: directory.path,
    size: directory.size,
    percentage: 0,
    isDir: true,
  }))

  if (rootFilesSize > 0) {
    values.push({
      id: `${root}:files`,
      label: 'Root Files',
      path: root,
      size: rootFilesSize,
      percentage: 0,
      isDir: false,
    })
  }

  const total = values.reduce((sum, item) => sum + item.size, 0)

  return values
    .map((item) => ({
      ...item,
      percentage: total > 0 ? (item.size / total) * 100 : 0,
    }))
    .sort((a, b) => b.size - a.size)
})

function canNavigateTile(item: TreemapItem): boolean {
  return item.isDir && item.path !== rootPath.value
}

function handleTileClick(item: TreemapItem) {
  if (props.navigating || !canNavigateTile(item)) return
  emit('navigate', item.path)
}

function layout(items: TreemapItem[]): Omit<Rect, 'colorIndex'>[] {
  if (items.length === 0) return []
  const total = items.reduce((sum, item) => sum + item.size, 0)
  if (total === 0) return []

  const result: Omit<Rect, 'colorIndex'>[] = []

  function partition(
    entries: TreemapItem[],
    x: number,
    y: number,
    width: number,
    height: number,
    horizontal: boolean,
  ) {
    if (entries.length === 0 || width <= 0 || height <= 0) return

    if (entries.length === 1) {
      const only = entries[0]
      if (only) {
        result.push({ item: only, x, y, width, height })
      }
      return
    }

    const sum = entries.reduce((value, item) => value + item.size, 0)
    let accumulated = 0
    let splitIndex = 1

    for (let index = 0; index < entries.length - 1; index++) {
      const entry = entries[index]
      if (!entry) break
      accumulated += entry.size
      if (accumulated >= sum / 2) {
        splitIndex = index + 1
        break
      }
    }

    const first = entries.slice(0, splitIndex)
    const second = entries.slice(splitIndex)
    const firstSize = first.reduce((value, item) => value + item.size, 0)
    const ratio = firstSize / sum

    if (horizontal) {
      const firstWidth = width * ratio
      partition(first, x, y, firstWidth, height, !horizontal)
      partition(second, x + firstWidth, y, width - firstWidth, height, !horizontal)
    } else {
      const firstHeight = height * ratio
      partition(first, x, y, width, firstHeight, !horizontal)
      partition(second, x, y + firstHeight, width, height - firstHeight, !horizontal)
    }
  }

  partition(
    items,
    0,
    0,
    VIEWBOX_WIDTH,
    VIEWBOX_HEIGHT,
    VIEWBOX_WIDTH >= VIEWBOX_HEIGHT,
  )

  return result
}

const rectangles = computed<Rect[]>(() =>
  layout(items.value).map((rectangle, idx) => ({
    ...rectangle,
    x: rectangle.x + GAP,
    y: rectangle.y + GAP,
    width: Math.max(rectangle.width - GAP * 2, 0),
    height: Math.max(rectangle.height - GAP * 2, 0),
    colorIndex: idx % TILE_PALETTE.length,
  })),
)

function fitsTitle(rect: Rect): boolean {
  return rect.width >= 50 && rect.height >= 28
}

function fitsMeta(rect: Rect): boolean {
  return rect.width >= 80 && rect.height >= 50
}

function paletteOf(index: number) {
  return (TILE_PALETTE[index % TILE_PALETTE.length] ?? TILE_PALETTE[0])!
}
</script>

<template>
  <section class="treemap" aria-labelledby="treemap-title">
    <header class="treemap__header">
      <div>
        <span class="treemap__eyebrow">STORAGE MAP</span>
        <h2 id="treemap-title">Where your storage is going</h2>
        <p class="treemap__path" :title="rootPath">
          <svg class="treemap__path-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z" />
          </svg>
          <code>{{ rootPath }}</code>
        </p>
      </div>

      <div class="treemap__summary">
        <div class="treemap__stat">
          <span class="treemap__stat-label">Total Size</span>
          <strong class="treemap__stat-value">{{ formatBytes(result.totalSize, 1) }}</strong>
        </div>
        <div class="treemap__stat">
          <span class="treemap__stat-label">Subfolders</span>
          <strong class="treemap__stat-value">{{items.filter(i => i.isDir).length}}</strong>
        </div>
      </div>
    </header>

    <div v-if="rectangles.length" class="treemap__canvas-wrapper">
      <svg class="treemap__svg" :viewBox="`0 0 ${VIEWBOX_WIDTH} ${VIEWBOX_HEIGHT}`" role="img"
        aria-label="Interactive directory storage map">
        <g v-for="rect in rectangles" :key="rect.item.id" class="treemap__tile" :class="{
          'treemap__tile--inert': !canNavigateTile(rect.item),
          'treemap__tile--hovered': hoveredItem?.id === rect.item.id,
        }" tabindex="0" role="button"
          :aria-label="`${rect.item.label}, ${formatBytes(rect.item.size)}, ${rect.item.percentage.toFixed(1)}%`"
          @click="handleTileClick(rect.item)" @keydown.enter.prevent="handleTileClick(rect.item)"
          @keydown.space.prevent="handleTileClick(rect.item)" @mouseenter="hoveredItem = rect.item"
          @mouseleave="hoveredItem = null">
          <!-- Tile Background -->
          <rect :x="rect.x" :y="rect.y" :width="rect.width" :height="rect.height" rx="6"
            :fill="paletteOf(rect.colorIndex).fill" :stroke="paletteOf(rect.colorIndex).border" stroke-width="1" />

          <!-- Native SVG Text Rendering (Reliable scaling across viewports) -->
          <g v-if="fitsTitle(rect)" class="treemap__label-group"
            :transform="`translate(${rect.x + 10}, ${rect.y + 18})`">
            <text class="treemap__tile-title" :fill="paletteOf(rect.colorIndex).text">
              {{ rect.item.label }}
            </text>

            <text v-if="fitsMeta(rect)" y="18" class="treemap__tile-meta" :fill="paletteOf(rect.colorIndex).muted">
              {{ formatBytes(rect.item.size, 1) }} • {{ rect.item.percentage.toFixed(1) }}%
            </text>
          </g>

          <title>
            {{ rect.item.label }} ({{ rect.item.path }})
            &#10;Size: {{ formatBytes(rect.item.size, 1) }} ({{ rect.item.percentage.toFixed(1) }}%)
            &#10;{{ canNavigateTile(rect.item) ? 'Click to navigate into directory' : 'Root directory files' }}
          </title>
        </g>
      </svg>
    </div>

    <div v-else class="treemap__empty">
      <svg class="treemap__empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
      <strong>No storage data available</strong>
      <span>Scan a directory containing files to inspect storage usage.</span>
    </div>

    <!-- Active Hover Detail Bar -->
    <div class="treemap__detail-bar">
      <template v-if="hoveredItem">
        <span class="treemap__detail-name">{{ hoveredItem.label }}</span>
        <span class="treemap__detail-path">{{ hoveredItem.path }}</span>
        <div class="treemap__detail-metrics">
          <strong>{{ formatBytes(hoveredItem.size, 1) }}</strong>
          <small>{{ hoveredItem.percentage.toFixed(1) }}%</small>
        </div>
      </template>
      <template v-else>
        <span class="treemap__detail-hint">Hover or focus a block to view path details</span>
      </template>
    </div>

    <!-- Quick Legend -->
    <div v-if="items.length" class="treemap__legend">
      <div v-for="(item, index) in items.slice(0, 6)" :key="item.id" class="treemap__legend-item"
        @mouseenter="hoveredItem = item" @mouseleave="hoveredItem = null">
        <span class="treemap__legend-dot" :style="{ background: paletteOf(index).border }" />
        <span class="treemap__legend-name">{{ item.label }}</span>
        <strong class="treemap__legend-value">{{ formatBytes(item.size, 1) }}</strong>
      </div>
    </div>
  </section>
</template>

<style scoped>
.treemap {
  --bg-surface: #ffffff;
  --bg-subtle: #f8fafc;
  --bg-hover: #f1f5f9;
  --border-color: #e2e8f0;
  --text-main: #0f172a;
  --text-muted: #64748b;
  --text-faint: #94a3b8;
  --accent: #2563eb;

  padding: 24px;
  border: 1px solid var(--border-color);
  border-radius: 14px;
  background: var(--bg-surface);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.02);
}

.treemap__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 20px;
  margin-bottom: 20px;
}

.treemap__eyebrow {
  color: var(--text-faint);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.treemap h2 {
  margin: 2px 0 0;
  color: var(--text-main);
  font-size: 18px;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.treemap__path {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin: 6px 0 0;
  color: var(--text-muted);
  font-size: 12px;
}

.treemap__path-icon {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}

.treemap__path code {
  padding: 2px 6px;
  border-radius: 4px;
  background: var(--bg-subtle);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 11px;
}

.treemap__summary {
  display: flex;
  gap: 20px;
}

.treemap__stat {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
}

.treemap__stat-label {
  color: var(--text-faint);
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
}

.treemap__stat-value {
  color: var(--text-main);
  font-size: 16px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.treemap__canvas-wrapper {
  overflow: hidden;
  border: 1px solid var(--border-color);
  border-radius: 10px;
  background: #0f172a;
}

.treemap__svg {
  display: block;
  width: 100%;
  height: auto;
}

.treemap__tile {
  cursor: pointer;
  outline: none;
}

.treemap__tile rect {
  transition: transform 150ms ease, filter 150ms ease, stroke-width 150ms ease;
  transform-origin: center;
}

.treemap__tile:hover rect,
.treemap__tile--hovered rect {
  filter: brightness(1.15);
  stroke-width: 2px;
}

.treemap__tile:focus-visible rect {
  stroke: #ffffff;
  stroke-width: 2px;
  filter: brightness(1.2);
}

.treemap__tile--inert {
  cursor: default;
}

.treemap__tile--inert rect {
  stroke-dasharray: 4 3;
}

.treemap__tile-title {
  font-size: 13px;
  font-weight: 600;
  pointer-events: none;
}

.treemap__tile-meta {
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  pointer-events: none;
}

.treemap__detail-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  min-height: 40px;
  margin-top: 12px;
  padding: 8px 14px;
  border-radius: 8px;
  background: var(--bg-subtle);
  font-size: 12px;
}

.treemap__detail-name {
  color: var(--text-main);
  font-weight: 600;
}

.treemap__detail-path {
  overflow: hidden;
  color: var(--text-muted);
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 11px;
}

.treemap__detail-metrics {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.treemap__detail-metrics strong {
  color: var(--text-main);
  font-variant-numeric: tabular-nums;
}

.treemap__detail-metrics small {
  color: var(--text-muted);
  font-variant-numeric: tabular-nums;
}

.treemap__detail-hint {
  color: var(--text-faint);
  font-style: italic;
}

.treemap__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 48px;
  border: 1px dashed var(--border-color);
  border-radius: 10px;
  background: var(--bg-subtle);
  color: var(--text-muted);
  font-size: 13px;
}

.treemap__empty-icon {
  width: 24px;
  height: 24px;
  color: var(--text-faint);
}

.treemap__legend {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 8px 16px;
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid var(--border-color);
}

.treemap__legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 6px;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 150ms ease;
}

.treemap__legend-item:hover {
  background: var(--bg-hover);
}

.treemap__legend-dot {
  width: 8px;
  height: 8px;
  border-radius: 2px;
  flex-shrink: 0;
}

.treemap__legend-name {
  overflow: hidden;
  color: var(--text-muted);
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
}

.treemap__legend-value {
  margin-left: auto;
  color: var(--text-main);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
}

@media (max-width: 640px) {
  .treemap__header {
    flex-direction: column;
  }

  .treemap__summary {
    width: 100%;
    justify-content: stroke;
  }

  .treemap__stat {
    align-items: flex-start;
  }

  .treemap__detail-bar {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
