<script setup lang="ts">
import type { ScanResult } from '@/types/scan'

defineProps<{
  result: ScanResult
}>()
</script>

<template>
  <section v-if="result.errors.length > 0" class="scan-warnings" role="status" aria-live="polite">
    <div class="scan-warnings__header">
      <strong>
        {{ result.errors.length }}
        {{ result.errors.length === 1 ? 'warning' : 'warnings' }}
      </strong>

      <span>
        Some paths could not be fully scanned.
      </span>
    </div>

    <ul class="scan-warnings__list">
      <li v-for="error in result.errors" :key="`${error.path}:${error.message}`">
        <strong>{{ error.path }}</strong>
        <span>{{ error.message }}</span>
      </li>
    </ul>
  </section>
</template>
