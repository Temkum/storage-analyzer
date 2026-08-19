<script setup lang="ts">
defineProps<{
  scanning: boolean
}>()
</script>

<template>
  <div class="dashboard">
    <header class="dashboard__header">
      <div>
        <p class="dashboard__eyebrow">SYSTEM ANALYZER</p>
        <h1>Disk Analyzer</h1>
        <p class="dashboard__description">
          Analyze disk usage and identify where your storage is being consumed.
        </p>
      </div>
    </header>

    <section class="dashboard__content">
      <slot />
    </section>
  </div>
</template>

<style scoped>
.dashboard {
  min-height: 100vh;
  padding: 32px;
  background: #f8fafc;
  color: #0f172a;
}

.dashboard__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 24px;
  max-width: 1200px;
  margin: 0 auto 32px;
}

.dashboard__eyebrow {
  margin: 0 0 6px;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.12em;
  color: #64748b;
}

.dashboard h1 {
  margin: 0;
  font-size: 32px;
  line-height: 1.1;
}

.dashboard__description {
  max-width: 600px;
  margin: 10px 0 0;
  color: #64748b;
}

.dashboard__status {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border: 1px solid #e2e8f0;
  border-radius: 999px;
  background: #fff;
  font-size: 13px;
  font-weight: 600;
}

.dashboard__status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #94a3b8;
}

.dashboard__status--active .dashboard__status-dot {
  background: #16a34a;
}

.dashboard__content {
  display: grid;
  gap: 24px;
  max-width: 1200px;
  margin: 0 auto;
}

@media (max-width: 700px) {
  .dashboard {
    padding: 20px;
  }

  .dashboard__header {
    flex-direction: column;
  }
}
</style>
