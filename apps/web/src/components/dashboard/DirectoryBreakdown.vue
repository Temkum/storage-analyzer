<script setup lang="ts">
import { computed } from 'vue'

import type { DirectorySize, ScanResult } from '@/types/scan'
import { formatBytes } from '@/utils/format'
import { basename, normalizePath } from '@/utils/paths'

const props = defineProps<{
  result: ScanResult
}>()

const directories = computed<DirectorySize[]>(() => {
  const root = normalizePath(props.result.rootPath)

  return props.result.directories
    .filter((directory) => normalizePath(directory.path) !== root)
    .slice()
    .sort((a, b) => b.size - a.size)
    .slice(0, 10)
})

const largestSize = computed(() => {
  const [first] = directories.value

  return first ? first.size : 0
})

const totalSize = computed(() => props.result.totalSize)

function fillWidth(size: number): number {
  if (largestSize.value === 0) {
    return 0
  }

  return (size / largestSize.value) * 100
}

function share(size: number): number {
  if (totalSize.value === 0) {
    return 0
  }

  return (size / totalSize.value) * 100
}
</script>

<template>
  <section class="directories">
    <div class="directories__header">
      <h2>Largest Directories</h2>
      <p>Top folders by recursive size under the scanned path.</p>
    </div>

    <p v-if="directories.length === 0" class="directories__empty">
      No directories found.
    </p>

    <ul v-else class="directories__list">
      <li v-for="directory in directories" :key="directory.path" class="directories__item">
        <div class="directories__row">
          <span class="directories__name" :title="directory.path">
            {{ basename(directory.path) }}
          </span>

          <span class="directories__share">
            {{ share(directory.size).toFixed(1) }}%
          </span>

          <strong>{{ formatBytes(directory.size) }}</strong>
        </div>

        <div
          class="directories__track"
          role="progressbar"
          :aria-valuenow="Math.round(fillWidth(directory.size))"
          aria-valuemin="0"
          aria-valuemax="100"
          :aria-label="`${basename(directory.path)}: ${formatBytes(directory.size)}`"
        >
          <div
            class="directories__fill"
            :style="{ width: `${fillWidth(directory.size)}%` }"
          />
        </div>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.directories {
  padding: 24px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #fff;
}

.directories__header h2 {
  margin: 0;
  font-size: 18px;
}

.directories__header p {
  margin: 4px 0 0;
  color: #64748b;
}

.directories__empty {
  color: #64748b;
}

.directories__list {
  display: grid;
  gap: 14px;
  margin: 18px 0 0;
  padding: 0;
  list-style: none;
}

.directories__row {
  display: flex;
  align-items: baseline;
  gap: 12px;
  min-width: 0;
  font-size: 13px;
}

.directories__name {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  color: #334155;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.directories__share {
  flex-shrink: 0;
  color: #94a3b8;
  font-size: 12px;
}

.directories__row strong {
  flex-shrink: 0;
  color: #0f172a;
  font-size: 13px;
}

.directories__track {
  height: 8px;
  margin-top: 6px;
  overflow: hidden;
  border-radius: 999px;
  background: #f1f5f9;
}

.directories__fill {
  height: 100%;
  border-radius: inherit;
  background: #0f172a;
  transition: width 300ms ease;
}
</style>