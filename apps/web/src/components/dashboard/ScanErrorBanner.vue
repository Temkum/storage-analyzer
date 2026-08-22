<script setup lang="ts">
defineProps<{
  message: string
}>()

const emit = defineEmits<{
  dismiss: []
}>()
</script>

<template>
  <section class="scan-error" role="alert">
    <div class="scan-error__icon" aria-hidden="true">
      <svg
        viewBox="0 0 24 24"
        width="18"
        height="18"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="8" x2="12" y2="12" />
        <line x1="12" y1="16" x2="12.01" y2="16" />
      </svg>
    </div>

    <div class="scan-error__body">
      <strong>Scan failed</strong>
      <p>{{ message }}</p>
    </div>

    <button
      class="scan-error__dismiss"
      type="button"
      aria-label="Dismiss error"
      @click="emit('dismiss')"
    >
      &times;
    </button>
  </section>
</template>

<style scoped>
.scan-error {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px 20px;
  border: 1px solid #fecaca;
  border-radius: 12px;
  background: #fef2f2;
  color: #991b1b;
}

.scan-error__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: #fee2e2;
  color: #dc2626;
}

.scan-error__body {
  flex: 1;
  min-width: 0;
}

.scan-error__body strong {
  display: block;
  font-size: 14px;
}

.scan-error__body p {
  margin: 2px 0 0;
  color: #b91c1c;
  font-size: 13px;
  line-height: 1.5;
  word-break: break-word;
}

.scan-error__dismiss {
  flex-shrink: 0;
  padding: 0;
  border: none;
  background: none;
  color: #dc2626;
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
  opacity: 0.7;
  transition: opacity 150ms ease;
}

.scan-error__dismiss:hover {
  opacity: 1;
}

.scan-error__dismiss:focus-visible {
  outline: 2px solid #dc2626;
  outline-offset: 2px;
  border-radius: 4px;
}
</style>
