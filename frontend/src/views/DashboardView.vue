<script setup lang="ts">
import { useAppStore } from "../stores/app";
const appStore = useAppStore();
</script>

<template>
  <h1>总览</h1>
  <p>store.title：{{ appStore.title }}</p>
</template>
