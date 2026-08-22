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
    // A common failure is the file having been deleted or moved since the
    // scan snapshot; surface that clearly instead of silently ignoring it.
    revealState.value = 'missing'
  } finally {
    revealBusy.value = false
  }
}
</script>

<template>
  <section class="largest-files">
    <div class="largest-files__header">
      <h2>Largest Files</h2>
      <p>Top files by size in this directory. Select a file to reveal it.</p>
    </div>

    <p v-if="rows.length === 0" class="largest-files__empty">
      No files found.
    </p>

    <ol v-else class="largest-files__list">
      <li v-for="(file, index) in rows" :key="file.path">
        <button type="button" class="largest-files__item"
          :class="{ 'largest-files__item--selected': selectedPath === file.path }"
          :aria-pressed="selectedPath === file.path" @click="toggleSelect(file)">
          <span class="largest-files__rank" aria-hidden="true">{{ index + 1 }}</span>

          <span class="largest-files__icon" :class="`largest-files__icon--${file.category}`" aria-hidden="true">
            {{ categoryGlyph(file.category) }}
          </span>

          <span class="largest-files__main">
            <span class="largest-files__name" :title="file.path">{{ file.name }}</span>
            <span class="largest-files__relpath">{{ file.relativePath }}</span>
          </span>

          <span class="largest-files__pct">
            {{ file.percentage.toFixed(1) }}%
          </span>

          <strong class="largest-files__size">{{ formatBytes(file.size) }}</strong>
        </button>
      </li>
    </ol>

    <div v-if="selected" class="largest-files__detail">
      <div class="largest-files__detail-head">
        <span class="largest-files__icon largest-files__icon--large"
          :class="`largest-files__icon--${selected.category}`" aria-hidden="true">
          {{ categoryGlyph(selected.category) }}
        </span>

        <div class="largest-files__detail-title">
          <strong>{{ selected.name }}</strong>
          <code>{{ selected.path }}</code>
        </div>
      </div>

      <dl class="largest-files__detail-meta">
        <div>
          <dt>Type</dt>
          <dd>{{ categoryLabel(selected.category) }}</dd>
        </div>

        <div>
          <dt>Size</dt>
          <dd>{{ formatBytes(selected.size) }}</dd>
        </div>

        <div>
          <dt>Share</dt>
          <dd>{{ selected.percentage.toFixed(1) }}% of scan</dd>
        </div>
      </dl>

      <div class="largest-files__detail-actions">
        <button type="button" class="largest-files__reveal" :disabled="scanning || revealBusy"
          @click="reveal(selected)">
          <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z" />
          </svg>

          {{ revealBusy ? 'Opening…' : 'Reveal in file manager' }}
        </button>

        <p v-if="revealState === 'missing'" class="largest-files__note" role="status">
          That file could not be found — it may have been deleted or moved since the scan.
        </p>
      </div>
    </div>
  </section>
</template>

<style scoped>
.largest-files {
  padding: 24px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #fff;
}

.largest-files__header h2 {
  margin: 0;
  font-size: 18px;
}

.largest-files__header p {
  margin: 4px 0 0;
  color: #64748b;
}

.largest-files__empty {
  color: #64748b;
}

.largest-files__list {
  display: grid;
  gap: 2px;
  margin: 16px 0 0;
  padding: 0;
  list-style: none;
}

.largest-files__item {
  display: flex;
  width: 100%;
  align-items: center;
  gap: 12px;
  min-width: 0;
  padding: 9px 10px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: inherit;
  font: inherit;
  text-align: left;
  cursor: pointer;
}

.largest-files__item:nth-child(odd) {
  background: #f8fafc;
}

.largest-files__item:hover {
  background: #f1f5f9;
}

.largest-files__item--selected {
  background: #eef2ff;
}

.largest-files__item:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: -2px;
}

.largest-files__rank {
  flex-shrink: 0;
  width: 18px;
  color: #94a3b8;
  font-size: 12px;
  font-weight: 700;
  text-align: right;
}

.largest-files__icon {
  display: inline-flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 7px;
  font-size: 15px;
}

.largest-files__icon--large {
  width: 34px;
  height: 34px;
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
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  gap: 1px;
}

.largest-files__name {
  overflow: hidden;
  color: #0f172a;
  font-size: 13px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.largest-files__relpath {
  overflow: hidden;
  color: #94a3b8;
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.largest-files__pct {
  flex-shrink: 0;
  color: #94a3b8;
  font-size: 12px;
}

.largest-files__size {
  flex-shrink: 0;
  color: #0f172a;
  font-size: 13px;
}

.largest-files__detail {
  margin-top: 16px;
  padding: 14px 16px;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  background: #f8fafc;
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
  gap: 2px;
}

.largest-files__detail-title strong {
  overflow: hidden;
  color: #0f172a;
  font-size: 14px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.largest-files__detail-title code {
  overflow: hidden;
  color: #64748b;
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.largest-files__detail-meta {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
  margin: 12px 0 0;
}

.largest-files__detail-meta div {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.largest-files__detail-meta dt {
  color: #94a3b8;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.largest-files__detail-meta dd {
  margin: 0;
  color: #334155;
  font-size: 13px;
  font-weight: 600;
}

.largest-files__detail-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 14px;
}

.largest-files__reveal {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 9px 16px;
  border: 1px solid #0f172a;
  border-radius: 9px;
  background: #0f172a;
  color: #fff;
  font: inherit;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition:
    background 150ms ease,
    transform 150ms ease,
    box-shadow 150ms ease;
}

.largest-files__reveal:hover:not(:disabled) {
  background: #1e293b;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(15, 23, 42, 0.18);
}

.largest-files__reveal:focus-visible {
  outline: 2px solid #2563eb;
  outline-offset: 2px;
}

.largest-files__reveal:disabled {
  background: #cbd5e1;
  color: #f8fafc;
  cursor: not-allowed;
}

.largest-files__note {
  margin: 0;
  color: #b45309;
  font-size: 12px;
}

@media (max-width: 700px) {
  .largest-files__detail-meta {
    grid-template-columns: 1fr;
  }

  .largest-files__pct {
    display: none;
  }
}
</style>
