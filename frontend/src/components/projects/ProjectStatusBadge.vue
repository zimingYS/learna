<script setup lang="ts">
import type {ProjectStatus} from "../../api/projects";
import {PROJECT_STATUS_LABELS} from "../../views/projects/project-rules";

defineProps<{ status: ProjectStatus }>();
</script>

<template>
  <span :class="['status-pill', `status-pill--${status}`]">
    {{ PROJECT_STATUS_LABELS[status] }}
  </span>
</template>

<style scoped>
.status-pill {
  padding: 4px 10px;

  border-radius: 999px;

  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
}

.status-pill--draft {
  background: #f1f3f8;
  color: var(--ink-3);
}

.status-pill--active {
  background: var(--brand-soft);
  color: var(--brand);
}

.status-pill--paused {
  background: #fff5e8;
  color: #b9761f;
}

.status-pill--completed {
  background: #e9f9f1;
  color: #12855a;
}

.status-pill--archived {
  background: #f2f4f8;
  color: #949cab;
}
</style>
