<script setup lang="ts">
import { computed } from 'vue'

import { FileType, type FileEntry, type ScanResult } from '@/types/scan'
import { formatBytes } from '@/utils/format'

const props = defineProps<{
  result: ScanResult
}>()

const largestFiles = computed<FileEntry[]>(() =>
  props.result.entries
    .filter((entry) => entry.type === FileType.File)
    .slice()
    .sort((a, b) => b.size - a.size)
    .slice(0, 10),
)
</script>

<template>
  <section class="largest-files">
    <div class="largest-files__header">
      <h2>Largest Files</h2>
      <p>Top files by size within the scanned path.</p>
    </div>

    <p v-if="largestFiles.length === 0" class="largest-files__empty">
      No files found.
    </p>

    <ol v-else class="largest-files__list">
      <li v-for="(file, index) in largestFiles" :key="file.path" class="largest-files__item">
        <span class="largest-files__rank" aria-hidden="true">{{ index + 1 }}</span>

        <span class="largest-files__path" :title="file.path">{{ file.path }}</span>

        <strong class="largest-files__size">{{ formatBytes(file.size) }}</strong>
      </li>
    </ol>
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
  align-items: center;
  gap: 12px;
  min-width: 0;
  padding: 9px 10px;
  border-radius: 8px;
}

.largest-files__item:nth-child(odd) {
  background: #f8fafc;
}

.largest-files__rank {
  flex-shrink: 0;
  width: 18px;
  color: #94a3b8;
  font-size: 12px;
  font-weight: 700;
  text-align: right;
}

.largest-files__path {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  color: #334155;
  font-size: 13px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.largest-files__size {
  flex-shrink: 0;
  color: #0f172a;
  font-size: 13px;
}
</style>