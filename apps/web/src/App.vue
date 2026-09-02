<script setup lang="ts">
import { ref } from 'vue'

import DashboardLayout from '@/components/layout/DashboardLayout.vue'
import DirectoryBreakdown from '@/components/dashboard/DirectoryBreakdown.vue'
import DirectoryBreadcrumb from '@/components/dashboard/DirectoryBreadcrumb.vue'
import DiskUsage from '@/components/dashboard/DiskUsage.vue'
import FileTypeBreakdown from '@/components/dashboard/FileTypeBreakdown.vue'
import LargestFiles from '@/components/dashboard/LargestFiles.vue'
import ScanControls from '@/components/dashboard/ScanControls.vue'
import ScanErrorBanner from '@/components/dashboard/ScanErrorBanner.vue'
import ScanSummary from '@/components/dashboard/ScanSummary.vue'
import ScanWarnings from '@/components/dashboard/ScanWarnings.vue'
import Treemap from '@/components/dashboard/Treemap.vue'
import Volumes from '@/components/dashboard/Volumes.vue'
import NetworkAnalyzer from '@/components/network/NetworkAnalyzer.vue'
import { useScanner } from '@/composables/useScanner'

const path = ref('/tmp/system-analyzer-test')
const breadcrumb = ref<string[]>([])

const {
  result,
  isScanning,
  scannedEntries,
  cancelled,
  error,
  clearError,
  scan,
  cancel,
} = useScanner()

async function handleScan() {
  breadcrumb.value = [path.value]
  await scan(path.value)
}

function handleCancel() {
  void cancel()
}

function handleDrillDown(targetPath: string) {
  if (isScanning.value) {
    return
  }

  path.value = targetPath

  const previous = breadcrumb.value[breadcrumb.value.length - 1]

  if (previous === targetPath) {
    // Re-scan the current directory without duplicating the breadcrumb.
    void scan(targetPath)
    return
  }

  breadcrumb.value = [...breadcrumb.value, targetPath]
  void scan(targetPath)
}

function handleBreadcrumbNavigate(targetPath: string) {
  if (isScanning.value) {
    return
  }

  const index = breadcrumb.value.findIndex((entry) => entry === targetPath)

  if (index === -1) {
    return
  }

  path.value = targetPath
  breadcrumb.value = breadcrumb.value.slice(0, index + 1)
  void scan(targetPath)
}
</script>

<template>
  <DashboardLayout :status="error ? 'error' : isScanning ? 'scanning' : 'idle'">
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

    <ScanControls v-model:path="path" :scanning="isScanning" :scanned-entries="scannedEntries" @scan="handleScan"
      @cancel="handleCancel" />

    <DirectoryBreadcrumb v-if="breadcrumb.length" :chain="breadcrumb" :scanning="isScanning"
      @navigate="handleBreadcrumbNavigate" />

    <section v-if="cancelled" class="scan-cancelled" role="status" aria-live="polite">
      <div class="scan-cancelled__icon" aria-hidden="true">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2"
          stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="16" x2="12" y2="12" />
          <line x1="12" y1="8" x2="12.01" y2="8" />
        </svg>
      </div>

      <strong>Scan cancelled</strong>
    </section>

    <ScanErrorBanner v-if="error" :message="error" @dismiss="clearError" />

    <template v-if="result">
      <ScanWarnings :result="result" />

      <ScanSummary :result="result" />

      <Treemap id="storage" :result="result" :navigating="isScanning" @navigate="handleDrillDown" />

      <div class="dashboard-grid dashboard-grid--two">
        <LargestFiles :result="result" :scanning="isScanning" />
        <DirectoryBreakdown :result="result" />
      </div>

      <FileTypeBreakdown :result="result" />

      <div class="dashboard-grid dashboard-grid--two">
        <DiskUsage :result="result" />
        <Volumes id="volumes" :result="result" />
      </div>
    </template>

    <section v-else class="empty-state">
      <div class="empty-state__icon">◈</div>

      <h2>Ready to analyze</h2>

      <p>
        Choose a directory above and start a scan to see your
        storage breakdown.
      </p>
    </section>

    <section id="network" class="network-section" aria-label="Network Analyzer">
      <div class="network-section__header">
        <p class="network-section__eyebrow">NETWORK ANALYSIS</p>

        <h2>Live network telemetry &amp; application attribution</h2>
      </div>

      <NetworkAnalyzer />
    </section>
  </DashboardLayout>
</template>

<style scoped>
.network-section {
  display: grid;
  gap: 16px;
  scroll-margin-top: 24px;
}

.network-section__header h2 {
  margin: 0;
  color: #0f172a;
  font-size: 20px;
  font-weight: 750;
  letter-spacing: -0.02em;
}

.network-section__eyebrow {
  margin: 0 0 6px;
  color: #64748b;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.12em;
}

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

.scan-cancelled {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  background: #f8fafc;
  color: #475569;
}

.scan-cancelled__icon {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: 50%;
  background: #e2e8f0;
  color: #64748b;
}

.scan-cancelled strong {
  font-size: 13px;
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
