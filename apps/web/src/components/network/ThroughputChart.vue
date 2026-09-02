<script setup lang="ts">
import { computed } from 'vue'

import { formatRate } from '@/utils/format'

export interface ChartPoint {
  timestamp: number
  bytesReceived: number
  bytesSent: number
}

const props = withDefaults(
  defineProps<{
    points: ChartPoint[]
    height?: number
  }>(),
  {
    height: 160,
  },
)

const WIDTH = 640

const rxPath = computed(() => seriesPath((point) => point.bytesReceived))
const txPath = computed(() => seriesPath((point) => point.bytesSent))

const rxAreaPath = computed(() => areaPath((point) => point.bytesReceived))

const peakRate = computed(() => {
  let peak = 0

  for (const point of props.points) {
    peak = Math.max(peak, point.bytesReceived, point.bytesSent)
  }

  return peak
})

function seriesPath(pick: (point: ChartPoint) => number): string | null {
  const points = props.points

  if (points.length < 2) {
    return null
  }

  const peak = peakRate.value || 1
  const first = points[0]
  const last = points[points.length - 1]

  if (!first || !last) {
    return null
  }

  const span = Math.max(last.timestamp - first.timestamp, 1)

  return points
    .map((point, index) => {
      const x = ((point.timestamp - first.timestamp) / span) * WIDTH
      const y = props.height - (pick(point) / peak) * (props.height - 8) - 4
      return `${index === 0 ? 'M' : 'L'}${x.toFixed(1)},${y.toFixed(1)}`
    })
    .join(' ')
}

function areaPath(pick: (point: ChartPoint) => number): string | null {
  const line = seriesPath(pick)

  if (!line) {
    return null
  }

  return `${line} L${WIDTH},${props.height} L0,${props.height} Z`
}

const peakLabel = computed(() => formatRate(peakRate.value))
</script>

<template>
  <div class="chart">
    <div v-if="points.length < 2" class="chart__empty">
      <p>Collecting samples…</p>
      <span>The chart fills in as the sampler observes traffic.</span>
    </div>

    <svg v-else class="chart__svg" :viewBox="`0 0 ${WIDTH} ${height}`" preserveAspectRatio="none" role="img"
      aria-label="Live network throughput">
      <path v-if="rxAreaPath" class="chart__area" :d="rxAreaPath" />
      <path v-if="rxPath" class="chart__line chart__line--rx" :d="rxPath" />
      <path v-if="txPath" class="chart__line chart__line--tx" :d="txPath" />
    </svg>

    <div class="chart__legend">
      <span class="chart__key chart__key--rx">RX</span>
      <span class="chart__key chart__key--tx">TX</span>
      <span class="chart__peak">peak {{ peakLabel }}</span>
    </div>
  </div>
</template>

<style scoped>
.chart {
  position: relative;
}

.chart__svg {
  display: block;
  width: 100%;
  height: 160px;
}

.chart__empty {
  display: flex;
  height: 160px;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border: 1px dashed #cbd5e1;
  border-radius: 10px;
  color: #64748b;
}

.chart__empty p {
  margin: 0;
  font-size: 13px;
  font-weight: 700;
}

.chart__empty span {
  margin-top: 4px;
  font-size: 12px;
}

.chart__area {
  fill: rgb(37 99 235 / 8%);
}

.chart__line {
  fill: none;
  stroke-width: 2;
  stroke-linejoin: round;
  stroke-linecap: round;
}

.chart__line--rx {
  stroke: #2563eb;
}

.chart__line--tx {
  stroke: #16a34a;
}

.chart__legend {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 8px;
  color: #64748b;
  font-size: 11px;
  font-weight: 700;
}

.chart__key::before {
  display: inline-block;
  width: 8px;
  height: 8px;
  margin-right: 5px;
  border-radius: 2px;
  content: '';
}

.chart__key--rx::before {
  background: #2563eb;
}

.chart__key--tx::before {
  background: #16a34a;
}

.chart__peak {
  margin-left: auto;
  color: #94a3b8;
}
</style>
