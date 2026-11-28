<script setup lang="ts">
import {statusTone} from "../../views/planning/planning-rules";

defineProps<{ label: string; status: string }>();
</script>

<template>
  <span :class="['plan-pill', `plan-pill--${statusTone(status)}`]">{{ label }}</span>
</template>

<style scoped>
.plan-pill {
  display: inline-flex;
  flex-shrink: 0;
  align-items: center;

  padding: 4px 10px;

  border-radius: 999px;

  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
}

.plan-pill--neutral {
  background: #f1f3f8;
  color: var(--ink-3);
}

.plan-pill--primary {
  background: var(--brand-soft);
  color: var(--brand);
}

.plan-pill--success {
  background: #e9f9f1;
  color: #12855a;
}

.plan-pill--warning {
  background: #fff5e8;
  color: #b9761f;
}

.plan-pill--danger {
  background: var(--rose-soft);
  color: #bf3a40;
}
</style>
