<script setup lang="ts">
import { computed } from 'vue'

import type { ScanResult } from '@/types/scan'
import { formatBytes } from '@/utils/format'
import { basename, isDirectChild, normalizePath } from '@/utils/paths'

interface TreemapItem {
  id: string
  label: string
  path: string
  size: number
  percentage: number
  depth: number
}

interface Rect {
  item: TreemapItem
  x: number
  y: number
  width: number
  height: number
}

const props = defineProps<{
  result: ScanResult
  navigating: boolean
}>()

const emit = defineEmits<{
  navigate: [path: string]
}>()

const VIEWBOX_WIDTH = 1000
const VIEWBOX_HEIGHT = 560
const GAP = 3

const items = computed<TreemapItem[]>(() => {
  const root = normalizePath(props.result.rootPath)

  const directories = props.result.directories
    .filter((directory) => isDirectChild(directory.path, root))
    .filter((directory) => directory.size > 0)

  const directorySize = directories.reduce(
    (total, directory) => total + directory.size,
    0,
  )

  const rootFilesSize = Math.max(
    props.result.totalSize - directorySize,
    0,
  )

  const values: TreemapItem[] = directories.map((directory) => ({
    id: directory.path,
    label: basename(directory.path),
    path: directory.path,
    size: directory.size,
    percentage: 0,
    depth: 0,
  }))

  if (rootFilesSize > 0) {
    values.push({
      id: `${root}:files`,
      label: 'Files in root',
      path: root,
      size: rootFilesSize,
      percentage: 0,
      depth: 0,
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
  return item.path !== normalizePath(props.result.rootPath)
}

function handleTileClick(item: TreemapItem) {
  if (props.navigating || !canNavigateTile(item)) {
    return
  }

  emit('navigate', item.path)
}

function layout(items: TreemapItem[]): Rect[] {
  if (items.length === 0) {
    return []
  }

  const total = items.reduce((sum, item) => sum + item.size, 0)

  if (total === 0) {
    return []
  }

  const result: Rect[] = []

  function partition(
    entries: TreemapItem[],
    x: number,
    y: number,
    width: number,
    height: number,
    horizontal: boolean,
  ) {
    if (entries.length === 0 || width <= 0 || height <= 0) {
      return
    }

    if (entries.length === 1) {
      const only = entries[0]

      if (only) {
        result.push({
          item: only,
          x,
          y,
          width,
          height,
        })
      }

      return
    }

    const sum = entries.reduce((value, item) => value + item.size, 0)

    let accumulated = 0
    let splitIndex = 1

    for (let index = 0; index < entries.length - 1; index += 1) {
      const entry = entries[index]

      if (!entry) {
        break
      }

      accumulated += entry.size

      if (accumulated >= sum / 2) {
        splitIndex = index + 1
        break
      }
    }

    const first = entries.slice(0, splitIndex)
    const second = entries.slice(splitIndex)

    const firstSize = first.reduce(
      (value, item) => value + item.size,
      0,
    )

    const ratio = firstSize / sum

    if (horizontal) {
      const firstWidth = width * ratio

      partition(
        first,
        x,
        y,
        firstWidth,
        height,
        !horizontal,
      )

      partition(
        second,
        x + firstWidth,
        y,
        width - firstWidth,
        height,
        !horizontal,
      )
    } else {
      const firstHeight = height * ratio

      partition(
        first,
        x,
        y,
        width,
        firstHeight,
        !horizontal,
      )

      partition(
        second,
        x,
        y + firstHeight,
        width,
        height - firstHeight,
        !horizontal,
      )
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

const rectangles = computed(() =>
  layout(items.value).map((rectangle) => ({
    ...rectangle,
    x: rectangle.x + GAP,
    y: rectangle.y + GAP,
    width: Math.max(rectangle.width - GAP * 2, 0),
    height: Math.max(rectangle.height - GAP * 2, 0),
  })),
)

function textFits(rectangle: Rect): boolean {
  return rectangle.width >= 120 && rectangle.height >= 58
}
</script>

<template>
  <section class="treemap">
    <div class="treemap__header">
      <div>
        <p class="treemap__eyebrow">STORAGE MAP</p>
        <h2>Where your storage is going</h2>
        <p>
          Each block is an immediate directory under the scanned path.
          Click a block to drill into it.
        </p>
      </div>

      <div class="treemap__total">
        <span>Total scanned</span>
        <strong>{{ formatBytes(result.totalSize, 1) }}</strong>
      </div>
    </div>

    <div v-if="rectangles.length" class="treemap__canvas">
      <svg viewBox="0 0 1000 560" preserveAspectRatio="none" role="img" aria-label="Storage usage treemap">
        <g v-for="(rectangle, index) in rectangles" :key="rectangle.item.id" class="treemap__tile" :class="{
          'treemap__tile--inert': !canNavigateTile(rectangle.item),
        }" role="button" :tabindex="canNavigateTile(rectangle.item) ? 0 : undefined"
          :aria-label="`Drill into ${rectangle.item.path}`"
          :aria-disabled="!canNavigateTile(rectangle.item) || navigating" @click="handleTileClick(rectangle.item)"
          @keydown.enter.prevent="handleTileClick(rectangle.item)"
          @keydown.space.prevent="handleTileClick(rectangle.item)">
          <rect :x="rectangle.x" :y="rectangle.y" :width="rectangle.width" :height="rectangle.height" rx="6"
            :class="`treemap__tile--${index % 6}`" />

          <foreignObject v-if="textFits(rectangle)" :x="rectangle.x + 12" :y="rectangle.y + 10"
            :width="Math.max(rectangle.width - 24, 1)" :height="Math.max(rectangle.height - 20, 1)">
            <div class="treemap__label">
              <strong>{{ rectangle.item.label }}</strong>
              <span>{{ formatBytes(rectangle.item.size, 1) }}</span>
              <small>
                {{ rectangle.item.percentage.toFixed(1) }}%
              </small>
            </div>
          </foreignObject>

          <title>
            {{ rectangle.item.path }}
            · {{ formatBytes(rectangle.item.size, 1) }}
            · {{ rectangle.item.percentage.toFixed(1) }}%
            · {{ canNavigateTile(rectangle.item) ? 'Click to drill down' : 'Files, not a directory' }}
          </title>
        </g>
      </svg>
    </div>

    <div v-else class="treemap__empty">
      <strong>No storage data available</strong>
      <span>Scan a directory containing files to see its storage map.</span>
    </div>

    <div v-if="items.length" class="treemap__legend">
      <div v-for="(item, index) in items.slice(0, 8)" :key="item.id" class="treemap__legend-item">
        <span class="treemap__legend-dot" :class="`treemap__legend-dot--${index % 6}`" />

        <span class="treemap__legend-name">
          {{ item.label }}
        </span>

        <strong>{{ formatBytes(item.size, 1) }}</strong>
      </div>
    </div>
  </section>
</template>

<style scoped>
.treemap {
  padding: 28px;
  border: 1px solid #e2e8f0;
  border-radius: 18px;
  background: #ffffff;
}

.treemap__header {
  display: flex;
  justify-content: space-between;
  gap: 24px;
  margin-bottom: 22px;
}

.treemap__eyebrow {
  margin: 0 0 5px;
  color: #64748b;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.12em;
}

.treemap h2 {
  margin: 0;
  color: #0f172a;
  font-size: 20px;
  line-height: 1.25;
}

.treemap__header p:not(.treemap__eyebrow) {
  margin: 6px 0 0;
  color: #64748b;
  font-size: 13px;
}

.treemap__total {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  white-space: nowrap;
}

.treemap__total span {
  color: #64748b;
  font-size: 12px;
}

.treemap__total strong {
  margin-top: 2px;
  color: #0f172a;
  font-size: 18px;
}

.treemap__canvas {
  width: 100%;
  overflow: hidden;
  border-radius: 10px;
  background: #f1f5f9;
}

.treemap__canvas svg {
  display: block;
  width: 100%;
  min-height: 420px;
}

.treemap__tile {
  cursor: pointer;
}

.treemap__tile rect {
  transition:
    filter 160ms ease,
    opacity 160ms ease;
}

.treemap__tile:hover rect {
  filter: brightness(0.92);
}

.treemap__tile:focus-visible {
  outline: 2px dashed #0f172a;
  outline-offset: 2px;
}

.treemap__tile--inert {
  cursor: default;
}

.treemap__tile--inert rect {
  stroke: #cbd5e1;
  stroke-dasharray: 6 4;
  stroke-width: 2;
}

.treemap__tile--inert:hover rect {
  filter: none;
}

.treemap__tile--0 {
  fill: #0f172a;
}

.treemap__tile--1 {
  fill: #1e293b;
}

.treemap__tile--2 {
  fill: #334155;
}

.treemap__tile--3 {
  fill: #475569;
}

.treemap__tile--4 {
  fill: #64748b;
}

.treemap__tile--5 {
  fill: #94a3b8;
}

.treemap__label {
  display: flex;
  flex-direction: column;
  color: #ffffff;
  font-family:
    Inter,
    -apple-system,
    BlinkMacSystemFont,
    'Segoe UI',
    sans-serif;
  line-height: 1.25;
  overflow: hidden;
}

.treemap__label strong {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 14px;
}

.treemap__label span {
  margin-top: 4px;
  font-size: 12px;
  opacity: 0.85;
}

.treemap__label small {
  margin-top: 2px;
  font-size: 11px;
  opacity: 0.7;
}

.treemap__empty {
  display: flex;
  min-height: 220px;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border: 1px dashed #cbd5e1;
  border-radius: 10px;
  background: #f8fafc;
  color: #64748b;
}

.treemap__empty strong {
  color: #334155;
}

.treemap__legend {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px 24px;
  margin-top: 20px;
}

.treemap__legend-item {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 8px;
  min-width: 0;
  font-size: 13px;
}

.treemap__legend-dot {
  width: 8px;
  height: 8px;
  border-radius: 2px;
}

.treemap__legend-dot--0 {
  background: #0f172a;
}

.treemap__legend-dot--1 {
  background: #1e293b;
}

.treemap__legend-dot--2 {
  background: #334155;
}

.treemap__legend-dot--3 {
  background: #475569;
}

.treemap__legend-dot--4 {
  background: #64748b;
}

.treemap__legend-dot--5 {
  background: #94a3b8;
}

.treemap__legend-name {
  min-width: 0;
  overflow: hidden;
  color: #475569;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.treemap__legend-item strong {
  color: #0f172a;
  font-size: 12px;
}

@media (max-width: 700px) {
  .treemap {
    padding: 20px;
  }

  .treemap__header {
    flex-direction: column;
  }

  .treemap__total {
    align-items: flex-start;
  }

  .treemap__canvas svg {
    min-height: 320px;
  }

  .treemap__legend {
    grid-template-columns: 1fr;
  }
}
</style>
