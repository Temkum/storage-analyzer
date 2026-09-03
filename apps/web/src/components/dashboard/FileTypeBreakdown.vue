<script setup lang="ts">
import { computed, ref } from 'vue'

import { FileType, type ScanResult } from '@/types/scan'
import {
  categoryGlyph,
  categoryLabel,
  fileCategory,
  fileExtension,
  type FileCategory,
} from '@/utils/fileCategory'
import { formatBytes } from '@/utils/format'

interface CategorySummary {
  category: FileCategory
  size: number
  count: number
  percentage: number
}

interface ExtensionSummary {
  extension: string
  category: FileCategory
  count: number
  size: number
  percentage: number
}

const props = defineProps<{
  result: ScanResult
}>()

const CATEGORY_COLORS: Record<FileCategory, string> = {
  document: '#3b82f6',
  image: '#22c55e',
  video: '#a855f7',
  audio: '#eab308',
  archive: '#64748b',
  code: '#0ea5e9',
  text: '#6366f1',
  disk: '#ef4444',
  other: '#94a3b8',
}

const totalSize = computed(() => props.result.totalSize)

const files = computed(() =>
  props.result.entries.filter((entry) => entry.type === FileType.File),
)

const categories = computed<CategorySummary[]>(() => {
  const groups = new Map<FileCategory, { size: number; count: number }>()

  for (const entry of files.value) {
    const category = fileCategory(entry.path)
    const existing = groups.get(category)

    if (existing) {
      existing.size += entry.size
      existing.count += 1
    } else {
      groups.set(category, { size: entry.size, count: 1 })
    }
  }

  const total = totalSize.value

  return Array.from(groups.entries())
    .map(([category, data]) => ({
      category,
      ...data,
      percentage: total > 0 ? (data.size / total) * 100 : 0,
    }))
    .sort((a, b) => b.size - a.size)
})

const extensions = computed<ExtensionSummary[]>(() => {
  const groups = new Map<string, ExtensionSummary>()

  for (const entry of files.value) {
    const extension = fileExtension(entry.path) || 'no extension'
    const existing = groups.get(extension)

    if (existing) {
      existing.count += 1
      existing.size += entry.size
    } else {
      groups.set(extension, {
        extension,
        category: fileCategory(entry.path),
        count: 1,
        size: entry.size,
        percentage: 0,
      })
    }
  }

  const total = totalSize.value

  return Array.from(groups.values())
    .map((group) => ({
      ...group,
      percentage: total > 0 ? (group.size / total) * 100 : 0,
    }))
    .sort((a, b) => b.size - a.size)
})

const activeCategory = ref<FileCategory | null>(null)

function toggleCategory(category: FileCategory) {
  activeCategory.value = activeCategory.value === category ? null : category
}

const visibleExtensions = computed<ExtensionSummary[]>(() => {
  const matches = activeCategory.value
    ? extensions.value.filter((item) => item.category === activeCategory.value)
    : extensions.value

  return matches.slice(0, 10)
})

interface DonutSegment extends CategorySummary {
  startAngle: number
}

const donutSegments = computed<DonutSegment[]>(() => {
  let cursor = 0

  return categories.value.map((category) => {
    const start = cursor
    cursor += category.percentage

    return {
      ...category,
      startAngle: Math.max(start * 3.6 - 90, -90),
    }
  })
})

function sliceArc(percentage: number): string {
  const visible = Math.max(percentage, 0.4)
  return `${visible} ${100 - visible}`
}

function colorOf(category: FileCategory): string {
  return CATEGORY_COLORS[category] ?? '#cbd5e1'
}

function extentLabel(extension: string): string {
  return extension === 'no extension' ? 'no ext' : `.${extension}`
}
</script>

<template>
  <section class="file-breakdown" aria-labelledby="breakdown-title">
    <header class="file-breakdown__header">
      <div>
        <h2 id="breakdown-title">File Type Analysis</h2>
        <p>Distribution of storage footprint by category and extension.</p>
      </div>

      <div class="file-breakdown__total-badge">
        <span class="file-breakdown__total-label">Total Volume</span>
        <strong class="file-breakdown__total-value">{{ formatBytes(totalSize) }}</strong>
      </div>
    </header>

    <div v-if="categories.length === 0" class="file-breakdown__empty">
      <svg class="file-breakdown__empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
      </svg>
      <span>No file records found in this directory.</span>
    </div>

    <template v-else>
      <div class="file-breakdown__overview">
        <div class="file-breakdown__chart-wrapper">
          <div class="file-breakdown__chart">
            <svg class="file-breakdown__donut" :class="{ 'file-breakdown__donut--filtering': activeCategory !== null }"
              viewBox="-50 -50 100 100" aria-hidden="true">
              <circle class="file-breakdown__track" cx="0" cy="0" r="15.915" pathLength="100" />

              <g v-for="segment in donutSegments" :key="segment.category" class="file-breakdown__segment-group"
                :style="{ transform: `rotate(${segment.startAngle}deg)` }">
                <circle class="file-breakdown__slice"
                  :class="{ 'file-breakdown__slice--active': activeCategory === segment.category }" cx="0" cy="0"
                  r="15.915" pathLength="100" :stroke="colorOf(segment.category)"
                  :stroke-dasharray="sliceArc(segment.percentage)" @click="toggleCategory(segment.category)" />
              </g>
            </svg>

            <div class="file-breakdown__center">
              <strong>{{ formatBytes(totalSize) }}</strong>
              <span>Total Usage</span>
            </div>
          </div>
        </div>

        <ul class="file-breakdown__categories" role="listbox" aria-label="File categories">
          <li v-for="summary in categories" :key="summary.category">
            <button type="button" class="file-breakdown__category"
              :class="{ 'file-breakdown__category--active': activeCategory === summary.category }" role="option"
              :aria-selected="activeCategory === summary.category" @click="toggleCategory(summary.category)">
              <span class="file-breakdown__swatch" :style="{ background: colorOf(summary.category) }"
                aria-hidden="true" />

              <span class="file-breakdown__category-name">
                <span>{{ categoryLabel(summary.category) }}</span>
                <span class="file-breakdown__glyph" aria-hidden="true">{{ categoryGlyph(summary.category) }}</span>
              </span>

              <span class="file-breakdown__category-count">
                {{ summary.count.toLocaleString() }} {{ summary.count === 1 ? 'file' : 'files' }}
              </span>

              <div class="file-breakdown__category-size">
                <strong>{{ formatBytes(summary.size) }}</strong>
                <small>{{ summary.percentage.toFixed(1) }}%</small>
              </div>
            </button>
          </li>
        </ul>
      </div>

      <div class="file-breakdown__extensions">
        <div class="file-breakdown__extensions-head">
          <h3>Top File Extensions</h3>

          <button v-if="activeCategory" type="button" class="file-breakdown__clear" @click="activeCategory = null">
            Filtered by <strong>{{ categoryLabel(activeCategory) }}</strong> • Clear filter
          </button>
        </div>

        <ul class="file-breakdown__extension-list">
          <li v-for="item in visibleExtensions" :key="item.extension" class="file-breakdown__extension">
            <div class="file-breakdown__ext-meta">
              <span class="file-breakdown__ext-badge" :style="{ color: colorOf(item.category) }">
                {{ extentLabel(item.extension) }}
              </span>
              <span class="file-breakdown__ext-count">
                {{ item.count.toLocaleString() }} {{ item.count === 1 ? 'file' : 'files' }}
              </span>
            </div>

            <div class="file-breakdown__ext-track" aria-hidden="true">
              <div class="file-breakdown__ext-fill" :style="{
                width: `${Math.max(item.percentage, 0.5)}%`,
                background: colorOf(item.category)
              }" />
            </div>

            <div class="file-breakdown__ext-metrics">
              <strong class="file-breakdown__ext-size">{{ formatBytes(item.size) }}</strong>
              <span class="file-breakdown__ext-pct">{{ item.percentage.toFixed(1) }}%</span>
            </div>
          </li>
        </ul>
      </div>
    </template>
  </section>
</template>

<style scoped>
.file-breakdown {
  --bg-surface: #ffffff;
  --bg-subtle: #f8fafc;
  --bg-hover: #f1f5f9;
  --border-color: #e2e8f0;
  --text-main: #0f172a;
  --text-muted: #64748b;
  --text-faint: #94a3b8;
  --accent-blue: #2563eb;

  padding: 24px;
  border: 1px solid var(--border-color);
  border-radius: 12px;
  background: var(--bg-surface);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.02);
}

.file-breakdown__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 24px;
}

.file-breakdown h2 {
  margin: 0;
  color: var(--text-main);
  font-size: 16px;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.file-breakdown__header p {
  margin: 2px 0 0;
  color: var(--text-muted);
  font-size: 13px;
}

.file-breakdown__total-badge {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
}

.file-breakdown__total-label {
  color: var(--text-faint);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.file-breakdown__total-value {
  color: var(--text-main);
  font-size: 18px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.file-breakdown__empty {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 32px;
  border: 1px dashed var(--border-color);
  border-radius: 8px;
  color: var(--text-muted);
  font-size: 13px;
}

.file-breakdown__empty-icon {
  width: 20px;
  height: 20px;
}

.file-breakdown__overview {
  display: grid;
  grid-template-columns: 240px minmax(0, 1fr);
  gap: 32px;
  align-items: center;
}

.file-breakdown__chart-wrapper {
  display: flex;
  justify-content: center;
}

.file-breakdown__chart {
  position: relative;
  width: 200px;
  height: 200px;
}

.file-breakdown__donut {
  width: 100%;
  height: 100%;
  transform: rotate(0deg);
}

.file-breakdown__track {
  fill: none;
  stroke: var(--bg-subtle);
  stroke-width: 6;
}

.file-breakdown__segment-group {
  transform-origin: center;
  transition: transform 300ms ease;
}

.file-breakdown__slice {
  fill: none;
  stroke-width: 6;
  cursor: pointer;
  transition: stroke-width 150ms ease, opacity 200ms ease, filter 150ms ease;
}

.file-breakdown__slice:hover {
  stroke-width: 7.5;
  filter: brightness(0.95);
}

.file-breakdown__donut--filtering .file-breakdown__slice {
  opacity: 0.25;
}

.file-breakdown__donut--filtering .file-breakdown__slice--active {
  opacity: 1;
  stroke-width: 7.5;
}

.file-breakdown__center {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  pointer-events: none;
  text-align: center;
}

.file-breakdown__center strong {
  color: var(--text-main);
  font-size: 16px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  line-height: 1.2;
}

.file-breakdown__center span {
  margin-top: 2px;
  color: var(--text-faint);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.file-breakdown__categories {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.file-breakdown__category {
  display: grid;
  grid-template-columns: auto minmax(120px, 1fr) auto auto;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 8px 12px;
  border: 1px solid transparent;
  border-radius: 8px;
  background: var(--bg-surface);
  color: inherit;
  font: inherit;
  text-align: left;
  cursor: pointer;
  transition: background-color 150ms ease, border-color 150ms ease;
}

.file-breakdown__category:hover {
  background: var(--bg-hover);
}

.file-breakdown__category--active {
  border-color: #bfdbfe;
  background: #eff6ff;
}

.file-breakdown__category:focus-visible {
  outline: 2px solid var(--accent-blue);
  outline-offset: -1px;
}

.file-breakdown__swatch {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.file-breakdown__category-name {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--text-main);
  font-size: 13px;
  font-weight: 500;
}

.file-breakdown__glyph {
  color: var(--text-faint);
  font-size: 12px;
}

.file-breakdown__category-count {
  color: var(--text-faint);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

.file-breakdown__category-size {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  line-height: 1.2;
}

.file-breakdown__category-size strong {
  color: var(--text-main);
  font-size: 12px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.file-breakdown__category-size small {
  color: var(--text-faint);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
}

.file-breakdown__extensions {
  margin-top: 24px;
  padding-top: 20px;
  border-top: 1px solid var(--border-color);
}

.file-breakdown__extensions-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}

.file-breakdown__extensions-head h3 {
  margin: 0;
  color: var(--text-main);
  font-size: 13px;
  font-weight: 600;
}

.file-breakdown__clear {
  padding: 4px 10px;
  border: 1px solid var(--border-color);
  border-radius: 999px;
  background: var(--bg-subtle);
  color: var(--text-muted);
  font: inherit;
  font-size: 11px;
  cursor: pointer;
  transition: background-color 150ms ease, color 150ms ease;
}

.file-breakdown__clear strong {
  color: var(--text-main);
}

.file-breakdown__clear:hover {
  background: var(--bg-hover);
  color: var(--text-main);
}

.file-breakdown__extension-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.file-breakdown__extension {
  display: grid;
  grid-template-columns: 160px minmax(100px, 1fr) 120px;
  align-items: center;
  gap: 16px;
  padding: 6px 10px;
  border-radius: 6px;
  background: var(--bg-subtle);
  font-size: 12px;
}

.file-breakdown__ext-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.file-breakdown__ext-badge {
  display: inline-block;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(15, 23, 42, 0.04);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 11px;
  font-weight: 700;
}

.file-breakdown__ext-count {
  color: var(--text-faint);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.file-breakdown__ext-track {
  height: 6px;
  overflow: hidden;
  border-radius: 999px;
  background: #e2e8f0;
}

.file-breakdown__ext-fill {
  height: 100%;
  border-radius: inherit;
  transition: width 300ms cubic-bezier(0.16, 1, 0.3, 1);
}

.file-breakdown__ext-metrics {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
}

.file-breakdown__ext-size {
  color: var(--text-main);
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.file-breakdown__ext-pct {
  width: 38px;
  color: var(--text-faint);
  text-align: right;
  font-size: 11px;
  font-variant-numeric: tabular-nums;
}

@media (max-width: 768px) {
  .file-breakdown__overview {
    grid-template-columns: 1fr;
    gap: 20px;
  }

  .file-breakdown__extension {
    grid-template-columns: minmax(120px, 1fr) 100px;
  }

  .file-breakdown__ext-track {
    display: none;
  }
}
</style>
