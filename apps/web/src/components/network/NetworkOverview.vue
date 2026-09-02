<script setup lang="ts">
import { computed } from 'vue'

import { formatRate } from '@/utils/format'

const props = defineProps<{
  bytesReceivedPerSecond: number
  bytesSentPerSecond: number
  connected: boolean
}>()

const rxLabel = computed(() => formatRate(props.bytesReceivedPerSecond))
const txLabel = computed(() => formatRate(props.bytesSentPerSecond))
</script>

<template>
  <section class="overview">
    <div class="overview__meter">
      <span class="overview__label overview__label--rx">RX</span>
      <strong class="overview__value">{{ connected ? rxLabel : '—' }}</strong>
    </div>

    <div class="overview__meter">
      <span class="overview__label overview__label--tx">TX</span>
      <strong class="overview__value">{{ connected ? txLabel : '—' }}</strong>
    </div>

    <p class="overview__note">
      RX/TX are total interface traffic. Application attribution on Linux covers only traffic
      attributable through /proc + TCP — the two numbers will legitimately differ.
    </p>
  </section>
</template>

<style scoped>
.overview {
  display: flex;
  align-items: center;
  gap: 32px;
  flex-wrap: wrap;
}

.overview__meter {
  display: flex;
  align-items: baseline;
  gap: 10px;
}

.overview__label {
  padding: 3px 9px;
  border-radius: 999px;
  color: #ffffff;
  font-size: 11px;
  font-weight: 800;
}

.overview__label--rx {
  background: #2563eb;
}

.overview__label--tx {
  background: #16a34a;
}

.overview__value {
  color: #0f172a;
  font-size: 26px;
  font-variant-numeric: tabular-nums;
}

.overview__note {
  flex: 1 1 260px;
  margin: 0;
  color: #94a3b8;
  font-size: 11px;
  line-height: 1.5;
}
</style>
