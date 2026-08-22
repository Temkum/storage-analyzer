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
  text: '#94a3b8',
  disk: '#ef4444',
  other: '#cbd5e1',
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

const largestExtensionSize = computed(() => {
  const [first] = visibleExtensions.value

  return first ? first.size : 0
})

// Donut segments. Each slice is drawn as its own circle rotated so its arc
// begins where the previous slice ended — avoids dashoffset sign ambiguity.

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
  const visible = Math.max(percentage, 0.2)
  return `${visible} ${100 - visible}`
}

function colorOf(category: FileCategory): string {
  return CATEGORY_COLORS[category] ?? '#cbd5e1'
}

function extentLabel(extension: string): string {
  return extension === 'no extension' ? 'no ext' : `.${extension}`
}

function fillWidth(size: number): number {
  if (largestExtensionSize.value === 0) {
    return 0
  }

  return (size / largestExtensionSize.value) * 100
}
</script>

<template>
  <section class="file-breakdown">
    <div class="file-breakdown__header">
      <div>
        <h2>File-Type Analysis</h2>
        <p>How scanned storage is distributed across file types.</p>
      </div>

      <strong class="file-breakdown__total">{{ formatBytes(totalSize) }}</strong>
    </div>

    <p v-if="categories.length === 0" class="file-breakdown__empty">
      No files found in this directory.
    </p>

    <template v-else>
      <div class="file-breakdown__overview">
        <div class="file-breakdown__chart">
          <svg class="file-breakdown__donut" :class="{ 'file-breakdown__donut--filtering': activeCategory }"
            viewBox="-50 -50 100 100" role="img" aria-label="Storage share by file type">
            <circle class="file-breakdown__track" cx="0" cy="0" r="15.915" pathLength="100" />

            <g v-for="segment in donutSegments" :key="segment.category"
              :transform="`rotate(${segment.startAngle} 0 0)`">
              <circle class="file-breakdown__slice"
                :class="{ 'file-breakdown__slice--active': activeCategory === segment.category }" cx="0" cy="0"
                r="15.915" pathLength="100" :stroke="colorOf(segment.category)"
                :stroke-dasharray="sliceArc(segment.percentage)" role="button" :tabindex="0"
                :aria-label="`${categoryLabel(segment.category)}: ${segment.percentage.toFixed(1)}%`"
                @click="toggleCategory(segment.category)" @keydown.enter.prevent="toggleCategory(segment.category)"
                @keydown.space.prevent="toggleCategory(segment.category)">
                <title>
                  {{ categoryLabel(segment.category) }}
                  · {{ formatBytes(segment.size) }}
                  · {{ segment.percentage.toFixed(1) }}%
                </title>
              </circle>
            </g>
          </svg>

          <div class="file-breakdown__center">
            <strong>{{ formatBytes(totalSize) }}</strong>
            <span>Scanned</span>
          </div>
        </div>

        <ul class="file-breakdown__categories">
          <li v-for="summary in categories" :key="summary.category">
            <button type="button" class="file-breakdown__category"
              :class="{ 'file-breakdown__category--active': activeCategory === summary.category }"
              @click="toggleCategory(summary.category)">
              <span class="file-breakdown__swatch" :style="{ background: colorOf(summary.category) }"
                aria-hidden="true" />

              <span class="file-breakdown__category-name" :title="categoryLabel(summary.category)">
                {{ categoryLabel(summary.category) }}
                <span aria-hidden="true" class="file-breakdown__glyph">
                  {{ categoryGlyph(summary.category) }}
                </span>
              </span>

              <span class="file-breakdown__category-count">
                {{ summary.count.toLocaleString() }}
              </span>

              <strong class="file-breakdown__category-size">
                {{ formatBytes(summary.size) }}
                <small>{{ summary.percentage.toFixed(1) }}%</small>
              </strong>
            </button>
          </li>
        </ul>
      </div>

      <div class="file-breakdown__extensions">
        <div class="file-breakdown__extensions-head">
          <h3>By extension</h3>

          <button v-if="activeCategory" type="button" class="file-breakdown__clear" @click="activeCategory = null">
            Showing {{ categoryLabel(activeCategory) }} · Show all
          </button>
        </div>

        <ul class="file-breakdown__extension-list">
          <li v-for="item in visibleExtensions" :key="item.extension" class="file-breakdown__extension">
            <span class="file-breakdown__ext-badge">{{ extentLabel(item.extension) }}</span>

            <span class="file-breakdown__ext-dot" :style="{ background: colorOf(item.category) }" aria-hidden="true" />

            <span class="file-breakdown__ext-count">
              {{ item.count.toLocaleString() }} file{{ item.count === 1 ? '' : 's' }}
            </span>

            <div class="file-breakdown__ext-track">
              <div class="file-breakdown__ext-fill" :style="{ width: `${fillWidth(item.size)}%` }" />
            </div>

            <strong class="file-breakdown__ext-size">{{ formatBytes(item.size) }}</strong>

            <span class="file-breakdown__ext-pct">{{ item.percentage.toFixed(1) }}%</span>
          </li>
        </ul>
      </div>
    </template>
  </section>
</template>

<style scoped>
.file-breakdown {
  padding: 24px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #fff;
}

.file-breakdown__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 20px;
}

.file-breakdown h2 {
  margin: 0;
  font-size: 18px;
}

.file-breakdown__header p {
  margin: 4px 0 0;
  color: #64748b;
}

.file-breakdown__total {
  color: #0f172a;
  font-size: 18px;
}

.file-breakdown__empty {
  color: #64748b;
}

.file-breakdown__overview {
  display: grid;
  grid-template-columns: minmax(220px, 320px) minmax(0, 1fr);
  gap: 32px;
  align-items: center;
}

.file-breakdown__chart {
  position: relative;
  max-width: 280px;
}

.file-breakdown__donut {
  display: block;
  width: 100%;
  height: auto;
}

.file-breakdown__track {
  fill: none;
  stroke: #f1f5f9;
  stroke-width: 7;
}

.file-breakdown__slice {
  fill: none;
  stroke-width: 7;
  cursor: pointer;
  transition: opacity 160ms ease, filter 160ms ease;
}

.file-breakdown__slice:hover,
.file-breakdown__slice:focus-visible {
  filter: brightness(0.9) drop-shadow(0 0 3px rgba(15, 23, 42, 0.35));
}

.file-breakdown__slice:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
  border-radius: 4px;
}

.file-breakdown__donut--filtering .file-breakdown__slice {
  opacity: 0.25;
}

.file-breakdown__donut--filtering .file-breakdown__slice--active {
  opacity: 1;
}

.file-breakdown__center {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  pointer-events: none;
  line-height: 1.2;
}

.file-breakdown__center strong {
  color: #0f172a;
  font-size: 16px;
}

.file-breakdown__center span {
  margin-top: 2px;
  color: #94a3b8;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.file-breakdown__categories {
  display: grid;
  gap: 6px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.file-breakdown__category {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto auto;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 8px 10px;
  border: 1px solid transparent;
  border-radius: 8px;
  background: transparent;
  color: inherit;
  font: inherit;
  text-align: left;
  cursor: pointer;
  transition:
    background 150ms ease,
    border-color 150ms ease;
}

.file-breakdown__category:hover {
  background: #f8fafc;
}

.file-breakdown__category--active {
  border-color: #e2e8f0;
  background: #f1f5f9;
}

.file-breakdown__category:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: -2px;
}

.file-breakdown__swatch {
  width: 10px;
  height: 10px;
  border-radius: 3px;
  flex-shrink: 0;
}

.file-breakdown__category-name {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  overflow: hidden;
  color: #334155;
  font-size: 13px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-breakdown__glyph {
  font-size: 13px;
}

.file-breakdown__category-count {
  color: #94a3b8;
  font-size: 12px;
}

.file-breakdown__category-size {
  display: inline-flex;
  flex-direction: column;
  align-items: flex-end;
  color: #0f172a;
  font-size: 12px;
  line-height: 1.3;
}

.file-breakdown__category-size small {
  color: #94a3b8;
  font-size: 11px;
  font-weight: 600;
}

.file-breakdown__extensions {
  margin-top: 26px;
  padding-top: 20px;
  border-top: 1px solid #eef2f7;
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
  color: #0f172a;
  font-size: 14px;
}

.file-breakdown__clear {
  padding: 5px 11px;
  border: 1px solid #e2e8f0;
  border-radius: 999px;
  background: #f8fafc;
  color: #2563eb;
  font: inherit;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.file-breakdown__clear:hover {
  background: #f1f5f9;
}

.file-breakdown__clear:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}

.file-breakdown__extension-list {
  display: grid;
  gap: 4px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.file-breakdown__extension {
  display: grid;
  grid-template-columns: auto auto minmax(90px, 1fr) minmax(80px, 2fr) auto auto;
  align-items: center;
  gap: 12px;
  padding: 7px 10px;
  border-radius: 8px;
  font-size: 13px;
}

.file-breakdown__extension:nth-child(odd) {
  background: #f8fafc;
}

.file-breakdown__ext-badge {
  min-width: 64px;
  padding: 3px 9px;
  border-radius: 999px;
  background: #e2e8f0;
  color: #334155;
  text-align: center;
  font-size: 12px;
  font-weight: 700;
}

.file-breakdown__ext-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.file-breakdown__ext-count {
  color: #94a3b8;
  font-size: 12px;
}

.file-breakdown__ext-track {
  height: 6px;
  overflow: hidden;
  border-radius: 999px;
  background: #eef2f7;
}

.file-breakdown__ext-fill {
  height: 100%;
  border-radius: inherit;
  background: #0f172a;
  transition: width 300ms ease;
}

.file-breakdown__ext-size {
  color: #0f172a;
  text-align: right;
  font-size: 12px;
}

.file-breakdown__ext-pct {
  min-width: 42px;
  color: #94a3b8;
  text-align: right;
  font-size: 12px;
}

@media (max-width: 900px) {
  .file-breakdown__overview {
    grid-template-columns: 1fr;
    gap: 24px;
  }

  .file-breakdown__chart {
    margin: 0 auto;
    max-width: 260px;
  }
}

@media (max-width: 600px) {
  .file-breakdown__extension {
    grid-template-columns: auto auto minmax(0, 1fr) auto;
  }

  .file-breakdown__ext-track {
    display: none;
  }
}
</style>
