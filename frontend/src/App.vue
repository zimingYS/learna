<script setup lang="ts">
import {RouterLink, RouterView} from "vue-router";
import {useAppStore} from "./stores/app"

const appStore = useAppStore()
</script>

<template>
  <div class="layout">
    <aside class="sidebar">
      <RouterLink to="/dashboard">总览</RouterLink>
      <RouterLink to="/records">学习记录</RouterLink>
      <RouterLink to="/roadmaps">路线图</RouterLink>
      <RouterLink to="/plans">计划</RouterLink>
      <RouterLink to="/reports">报告</RouterLink>
      <RouterLink to="/settings">设置</RouterLink>
    </aside>

    <div class="main">
      <header class="topbar">
        {{ appStore.title }}
      </header>

      <main class="content">
        <RouterView/>
      </main>
    </div>
  </div>
</template>

<style scoped>
.layout {
  display: flex;
  height: 100vh;
}

.sidebar {
  width: 200px;
  padding: 12px;
}

.main {
  display: flex;
  flex: 1;
  flex-direction: column;
}

.topbar {
  display: flex;
  justify-content: center;
}

.content {
  flex: 1;
}
</style>
