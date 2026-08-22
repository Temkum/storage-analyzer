<script setup lang="ts">
import type { ScanResult } from '@/types/scan'
import { formatBytes } from '@/utils/format'

defineProps<{
  result: ScanResult
}>()
</script>

<template>
  <section class="scan-summary">
    <div class="scan-summary__header">
      <h2>Scan Summary</h2>
      <p>Overview of the completed disk scan.</p>
    </div>

    <div class="scan-summary__grid">
      <article class="scan-summary__stat">
        <span>Total Size</span>
        <strong>{{ formatBytes(result.totalSize) }}</strong>
      </article>

      <article class="scan-summary__stat">
        <span>Files</span>
        <strong>{{ result.fileCount.toLocaleString() }}</strong>
      </article>

      <article class="scan-summary__stat">
        <span>Directories</span>
        <strong>{{ result.directoryCount.toLocaleString() }}</strong>
      </article>

      <article class="scan-summary__stat">
        <span>Scan Time</span>
        <strong>{{ result.durationMs.toLocaleString() }} ms</strong>
      </article>
    </div>
  </section>
</template>

<style scoped>
.scan-summary {
  padding: 24px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #fff;
}

.scan-summary__header h2 {
  margin: 0;
  font-size: 18px;
}

.scan-summary__header p {
  margin: 4px 0 0;
  color: #64748b;
}

.scan-summary__grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 16px;
  margin-top: 18px;
}

.scan-summary__stat {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 16px;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  background: #f8fafc;
}

.scan-summary__stat span {
  color: #64748b;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.scan-summary__stat strong {
  color: #0f172a;
  font-size: 18px;
}

@media (max-width: 900px) {
  .scan-summary__grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 500px) {
  .scan-summary__grid {
    grid-template-columns: 1fr;
  }
}
</style>