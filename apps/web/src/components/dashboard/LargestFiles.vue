<script setup lang="ts">
import { computed, ref } from 'vue'

import { revealInFileManager } from '@/services/analyzer'
import { FileType, type FileEntry, type ScanResult } from '@/types/scan'
import {
  categoryGlyph,
  categoryLabel,
  fileCategory,
} from '@/utils/fileCategory'
import { formatBytes } from '@/utils/format'
import { basename, normalizePath, relativePath } from '@/utils/paths'

interface FileRow extends FileEntry {
  name: string
  relativePath: string
  category: ReturnType<typeof fileCategory>
  percentage: number
}

const props = defineProps<{
  result: ScanResult
  scanning: boolean
}>()

const rootPath = computed(() => normalizePath(props.result.rootPath))
const totalSize = computed(() => props.result.totalSize)

const rows = computed<FileRow[]>(() => {
  const root = rootPath.value
  const total = totalSize.value

  return props.result.entries
    .filter((entry) => entry.type === FileType.File)
    .slice()
    .sort((a, b) => b.size - a.size)
    .slice(0, 10)
    .map((entry) => ({
      ...entry,
      name: basename(entry.path),
      relativePath: relativePath(entry.path, root),
      category: fileCategory(entry.path),
      percentage: total > 0 ? (entry.size / total) * 100 : 0,
    }))
})

const selectedPath = ref<string | null>(null)
const revealBusy = ref(false)
const revealState = ref<'idle' | 'missing'>('idle')

const selected = computed<FileRow | null>(
  () => rows.value.find((row) => row.path === selectedPath.value) ?? null,
)

function toggleSelect(row: FileRow) {
  revealState.value = 'idle'
  selectedPath.value = selectedPath.value === row.path ? null : row.path
}

async function reveal(entry: FileRow) {
  revealBusy.value = true
  revealState.value = 'idle'

  try {
    await revealInFileManager(entry.path)
  } catch {
    revealState.value = 'missing'
  } finally {
    revealBusy.value = false
  }
}
</script>

<template>
  <section class="largest-files" aria-labelledby="largest-files-title">
    <header class="largest-files__header">
      <h2 id="largest-files-title">Largest Files</h2>
      <p>Top 10 files by disk usage. Select an item to inspect or manage.</p>
    </header>

    <div v-if="rows.length === 0" class="largest-files__empty">
      <svg class="largest-files__empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z" />
        <polyline points="13 2 13 9 20 9" />
      </svg>
      <span>No files detected in this scan workspace.</span>
    </div>

    <div v-else class="largest-files__content">
      <ol class="largest-files__list" role="listbox" aria-label="Largest files list">
        <li v-for="(file, index) in rows" :key="file.path">
          <button type="button" class="largest-files__item"
            :class="{ 'largest-files__item--selected': selectedPath === file.path }" role="option"
            :aria-selected="selectedPath === file.path" @click="toggleSelect(file)">
            <!-- Visual scale context bar -->
            <span class="largest-files__bar" :style="{ width: `${Math.max(file.percentage, 1)}%` }"
              aria-hidden="true" />

            <span class="largest-files__rank" aria-hidden="true">{{ index + 1 }}</span>

            <span class="largest-files__icon" :class="`largest-files__icon--${file.category}`" aria-hidden="true">
              {{ categoryGlyph(file.category) }}
            </span>

            <span class="largest-files__main">
              <span class="largest-files__name" :title="file.path">{{ file.name }}</span>
              <span class="largest-files__relpath">{{ file.relativePath }}</span>
            </span>

            <span class="largest-files__metrics">
              <span class="largest-files__pct">{{ file.percentage.toFixed(1) }}%</span>
              <strong class="largest-files__size">{{ formatBytes(file.size) }}</strong>
            </span>
          </button>
        </li>
      </ol>

      <Transition name="expand">
        <div v-if="selected" class="largest-files__detail">
          <div class="largest-files__detail-head">
            <span class="largest-files__icon largest-files__icon--large"
              :class="`largest-files__icon--${selected.category}`" aria-hidden="true">
              {{ categoryGlyph(selected.category) }}
            </span>

            <div class="largest-files__detail-title">
              <strong>{{ selected.name }}</strong>
              <code :title="selected.path">{{ selected.path }}</code>
            </div>
          </div>

          <dl class="largest-files__detail-meta">
            <div>
              <dt>Category</dt>
              <dd>{{ categoryLabel(selected.category) }}</dd>
            </div>
            <div>
              <dt>Size</dt>
              <dd>{{ formatBytes(selected.size) }}</dd>
            </div>
            <div>
              <dt>Disk Impact</dt>
              <dd>{{ selected.percentage.toFixed(2) }}%</dd>
            </div>
          </dl>

          <div class="largest-files__detail-actions">
            <button type="button" class="largest-files__reveal" :disabled="scanning || revealBusy"
              @click="reveal(selected)">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.2"
                stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
              </svg>
              <span>{{ revealBusy ? 'Locating...' : 'Reveal in File Manager' }}</span>
            </button>

            <p v-if="revealState === 'missing'" class="largest-files__note" role="status">
              File not found. It may have been moved or deleted since scanning.
            </p>
          </div>
        </div>
      </Transition>
    </div>
  </section>
</template>

<style scoped>
.largest-files {
  --bg-surface: #ffffff;
  --bg-subtle: #f8fafc;
  --bg-hover: #f1f5f9;
  --bg-selected: #eff6ff;
  --border-color: #e2e8f0;
  --text-main: #0f172a;
  --text-muted: #64748b;
  --text-faint: #94a3b8;
  --accent-color: #2563eb;

  min-width: 0;
  padding: 20px;
  border: 1px solid var(--border-color);
  border-radius: 12px;
  background: var(--bg-surface);
  box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.02);
}

.largest-files__header h2 {
  margin: 0;
  color: var(--text-main);
  font-size: 16px;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.largest-files__header p {
  margin: 2px 0 0;
  color: var(--text-muted);
  font-size: 13px;
}

.largest-files__empty {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 16px;
  padding: 24px;
  border: 1px dashed var(--border-color);
  border-radius: 8px;
  color: var(--text-muted);
  font-size: 13px;
}

.largest-files__empty-icon {
  width: 18px;
  height: 18px;
}

.largest-files__list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin: 16px 0 0;
  padding: 0;
  list-style: none;
}

.largest-files__item {
  position: relative;
  display: flex;
  width: 100%;
  align-items: center;
  gap: 12px;
  min-width: 0;
  padding: 8px 12px;
  border: 1px solid transparent;
  border-radius: 8px;
  background: var(--bg-surface);
  color: inherit;
  font: inherit;
  text-align: left;
  cursor: pointer;
  overflow: hidden;
  transition: border-color 150ms ease, background-color 150ms ease;
}

.largest-files__bar {
  position: absolute;
  top: 0;
  bottom: 0;
  left: 0;
  background: rgba(37, 99, 235, 0.04);
  pointer-events: none;
  transition: width 300ms ease;
}

.largest-files__item:hover {
  background: var(--bg-hover);
}

.largest-files__item--selected {
  border-color: #bfdbfe;
  background: var(--bg-selected);
}

.largest-files__item:focus-visible {
  outline: 2px solid var(--accent-color);
  outline-offset: -1px;
}

.largest-files__rank {
  position: relative;
  z-index: 1;
  flex-shrink: 0;
  width: 16px;
  color: var(--text-faint);
  font-size: 11px;
  font-weight: 700;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.largest-files__icon {
  position: relative;
  z-index: 1;
  display: inline-flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 6px;
  font-size: 14px;
}

.largest-files__icon--large {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  font-size: 18px;
}

.largest-files__icon--document {
  background: #eff6ff;
}

.largest-files__icon--image {
  background: #f0fdf4;
}

.largest-files__icon--video {
  background: #fdf4ff;
}

.largest-files__icon--audio {
  background: #fefce8;
}

.largest-files__icon--archive {
  background: #f1f5f9;
}

.largest-files__icon--code {
  background: #e0f2fe;
}

.largest-files__icon--text {
  background: #f8fafc;
}

.largest-files__icon--disk {
  background: #fee2e2;
}

.largest-files__icon--other {
  background: #f1f5f9;
}

.largest-files__main {
  position: relative;
  z-index: 1;
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
}

.largest-files__name {
  overflow: hidden;
  color: var(--text-main);
  font-size: 13px;
  font-weight: 500;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.largest-files__relpath {
  overflow: hidden;
  color: var(--text-muted);
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.largest-files__metrics {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.largest-files__pct {
  color: var(--text-faint);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

.largest-files__size {
  color: var(--text-main);
  font-size: 13px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.largest-files__detail {
  margin-top: 12px;
  padding: 16px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--bg-subtle);
}

.largest-files__detail-head {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.largest-files__detail-title {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
}

.largest-files__detail-title strong {
  overflow: hidden;
  color: var(--text-main);
  font-size: 14px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.largest-files__detail-title code {
  overflow: hidden;
  color: var(--text-muted);
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.largest-files__detail-meta {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
  margin: 12px 0 0;
  padding-top: 12px;
  border-top: 1px solid var(--border-color);
}

.largest-files__detail-meta dt {
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 500;
}

.largest-files__detail-meta dd {
  margin: 2px 0 0;
  color: var(--text-main);
  font-size: 12px;
  font-weight: 600;
}

.largest-files__detail-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 12px;
}

.largest-files__reveal {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: 1px solid #0f172a;
  border-radius: 6px;
  background: #0f172a;
  color: #ffffff;
  font: inherit;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 150ms ease;
}

.largest-files__reveal:hover:not(:disabled) {
  background: #1e293b;
}

.largest-files__reveal:focus-visible {
  outline: 2px solid var(--accent-color);
  outline-offset: 2px;
}

.largest-files__reveal:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.largest-files__note {
  margin: 0;
  color: #dc2626;
  font-size: 12px;
}

/* Vue Animations */
.expand-enter-active,
.expand-leave-active {
  transition: all 200ms cubic-bezier(0.16, 1, 0.3, 1);
  max-height: 200px;
  opacity: 1;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  max-height: 0;
  opacity: 0;
  margin-top: 0;
  padding-top: 0;
  padding-bottom: 0;
}

@media (max-width: 600px) {
  .largest-files__pct {
    display: none;
  }

  .largest-files__detail-meta {
    grid-template-columns: repeat(1, minmax(0, 1fr));
  }
}
</style>
