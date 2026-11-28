<script setup lang="ts">
import StatusPill from "../ui/StatusPill.vue";
import {statusTone} from "../../views/planning/planning-rules";

defineProps<{ label: string; status: string }>();
</script>

<template>
  <StatusPill :label="label" :tone="statusTone(status)"/>
</template>
