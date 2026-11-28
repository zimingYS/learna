<script setup lang="ts">
// 状态徽标的公共外观：配色只在这里定义一次，各域只负责把状态映射成 tone
defineProps<{
  label: string;
  tone: "neutral" | "primary" | "success" | "warning" | "danger";
}>();
</script>

<template>
  <span :class="['status-pill', `status-pill--${tone}`]">{{ label }}</span>
</template>

<style scoped>
.status-pill {
  display: inline-flex;
  flex-shrink: 0;
  align-items: center;

  padding: 4px 10px;

  border-radius: 999px;

  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
}

.status-pill--neutral {
  background: #f1f3f8;
  color: var(--ink-3);
}

.status-pill--primary {
  background: var(--brand-soft);
  color: var(--brand);
}

.status-pill--success {
  background: #e9f9f1;
  color: #12855a;
}

.status-pill--warning {
  background: #fff5e8;
  color: #b9761f;
}

.status-pill--danger {
  background: var(--rose-soft);
  color: #bf3a40;
}
</style>
