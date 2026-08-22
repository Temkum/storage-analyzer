<script setup lang="ts">
import { ref } from 'vue'

import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import DirectoryBreakdown from '@/components/dashboard/DirectoryBreakdown.vue'
import DiskUsage from '@/components/dashboard/DiskUsage.vue'
import FileTypeBreakdown from '@/components/dashboard/FileTypeBreakdown.vue'
import LargestFiles from '@/components/dashboard/LargestFiles.vue'
import ScanControls from '@/components/dashboard/ScanControls.vue'
import ScanErrorBanner from '@/components/dashboard/ScanErrorBanner.vue'
import ScanSummary from '@/components/dashboard/ScanSummary.vue'
import ScanWarnings from '@/components/dashboard/ScanWarnings.vue'
import StorageUsage from '@/components/dashboard/StorageUsage.vue'
import Treemap from '@/components/dashboard/Treemap.vue'
import Volumes from '@/components/dashboard/Volumes.vue'
import { useScanner } from '@/composables/useScanner'

const path = ref('/tmp/system-analyzer-test')

const {
  result,
  isScanning,
  scannedEntries,
  error,
  clearError,
  scan,
} = useScanner()

async function handleScan() {
  await scan(path.value)
}
</script>

<template>
  <DashboardLayout :scanning="isScanning">
    <section id="overview" class="hero">
      <div>
        <p class="hero__eyebrow">DISK ANALYSIS</p>

        <h2>
          Understand what is<br />
          using your storage
        </h2>

        <p class="hero__description">
          Scan a directory to identify large files, storage-heavy
          directories, and file-type distribution.
        </p>
      </div>

      <div v-if="result" class="hero__path">
        <span>Last scanned</span>
        <strong>{{ result.rootPath }}</strong>
      </div>
    </section>

    <ScanControls v-model:path="path" :scanning="isScanning" :scanned-entries="scannedEntries" @scan="handleScan" />

    <ScanErrorBanner v-if="error" :message="error" @dismiss="clearError" />

    <template v-if="result">
      <ScanWarnings :result="result" />

      <ScanSummary :result="result" />

      <Treemap id="storage" :result="result" />

      <div class="dashboard-grid dashboard-grid--two">
        <DiskUsage :result="result" />
        <StorageUsage :result="result" />
      </div>

      <div class="dashboard-grid dashboard-grid--two">
        <LargestFiles :result="result" />
        <DirectoryBreakdown :result="result" />
      </div>

      <FileTypeBreakdown :result="result" />

      <Volumes id="volumes" :result="result" />
    </template>

    <section v-else class="empty-state">
      <div class="empty-state__icon">◈</div>

      <h2>Ready to analyze</h2>

      <p>
        Choose a directory above and start a scan to see your
        storage breakdown.
      </p>
    </section>
  </DashboardLayout>
</template>

<style scoped>
.hero {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 32px;
  scroll-margin-top: 24px;
}

.hero__eyebrow {
  margin: 0 0 8px;
  color: #64748b;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.12em;
}

.hero h2 {
  margin: 0;
  color: #0f172a;
  font-size: clamp(28px, 4vw, 42px);
  font-weight: 750;
  letter-spacing: -0.035em;
  line-height: 1.05;
}

.hero__description {
  max-width: 600px;
  margin: 14px 0 0;
  color: #64748b;
  font-size: 14px;
}

.hero__path {
  max-width: 360px;
  padding: 13px 15px;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  background: #ffffff;
}

.hero__path span,
.hero__path strong {
  display: block;
}

.hero__path span {
  margin-bottom: 4px;
  color: #94a3b8;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
}

.hero__path strong {
  overflow: hidden;
  color: #334155;
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dashboard-grid {
  display: grid;
  gap: 24px;
}

.dashboard-grid--two {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.empty-state {
  display: flex;
  min-height: 320px;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px;
  border: 1px dashed #cbd5e1;
  border-radius: 18px;
  background: #ffffff;
  text-align: center;
  scroll-margin-top: 24px;
}

.empty-state__icon {
  display: grid;
  width: 48px;
  height: 48px;
  place-items: center;
  margin-bottom: 16px;
  border-radius: 12px;
  background: #f1f5f9;
  color: #334155;
  font-size: 22px;
}

.empty-state h2 {
  margin: 0;
  color: #0f172a;
  font-size: 18px;
}

.empty-state p {
  max-width: 420px;
  margin: 7px 0 0;
  color: #64748b;
  font-size: 13px;
}

@media (max-width: 900px) {
  .hero {
    align-items: flex-start;
    flex-direction: column;
  }

  .hero__path {
    width: 100%;
    max-width: none;
  }

  .dashboard-grid--two {
    grid-template-columns: 1fr;
  }
}
</style>
