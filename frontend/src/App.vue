<script setup lang="ts">
import {RouterLink, RouterView, useRoute, useRouter} from "vue-router";
import {useAppStore} from "./stores/app"
import {useAuthStore} from "./stores/auth.ts";

const appStore = useAppStore();
const authStore = useAuthStore();

const route = useRoute();
const router = useRouter();

// 退出登录
async function handleLogout(): Promise<void> {
  authStore.logout();
  await router.push("/login");
}
</script>

<template>
  <div class="layout" :class="{ 'layout--blank': route.meta.blank }">
    <!-- 侧栏 -->
    <template v-if="!route.meta.blank">
      <aside class="sidebar">
        <nav class="sidebar-nav">
          <RouterLink to="/dashboard">总览</RouterLink>
          <RouterLink to="/records">学习记录</RouterLink>
          <RouterLink to="/roadmaps">路线图</RouterLink>
          <RouterLink to="/plans">计划</RouterLink>
          <RouterLink to="/reports">报告</RouterLink>
          <RouterLink to="/settings">设置</RouterLink>
        </nav>

        <!-- 登录状态 -->
        <div class="sidebar-auth">
          <template v-if="authStore.isLoggedIn">
            <!-- 名称显示 -->
            <span class="user-name">
              {{ authStore.user?.display_name }}
            </span>

            <button type="button" class="logout-btn" @click="handleLogout()">退出</button>
          </template>

          <RouterLink v-else to="/login">登录</RouterLink>
        </div>
      </aside>
    </template>

    <div class="main">
      <!-- 顶部导航 -->
      <template v-if="!route.meta.blank">
        <header class="topbar">
          {{ appStore.title }}
        </header>
      </template>

      <main class="content">
        <RouterView />
      </main>
    </div>
  </div>
</template>

<style scoped>
/* ==============================
   Global
============================== */

:global(*) {
  box-sizing: border-box;
}

:global(html),
:global(body),
:global(#app) {
  min-width: 320px;
  min-height: 100%;
  margin: 0;
}

:global(body) {
  font-family:
    Inter,
    -apple-system,
    BlinkMacSystemFont,
    "Segoe UI",
    "Microsoft YaHei",
    sans-serif;
  color: #252b3b;
  background: #f5f6fa;
}

:global(button),
:global(input) {
  font: inherit;
}

/* ==============================
   App Layout
============================== */

.layout {
  display: flex;
  width: 100%;
  min-height: 100vh;
  background: #f5f6fa;
}

/* ==============================
   Sidebar
============================== */

.sidebar {
  display: flex;
  flex-shrink: 0;
  flex-direction: column;

  width: 232px;
  padding: 26px 16px 16px;

  background: #ffffff;
  border-right: 1px solid #eaecf2;
}

/* 品牌名称 */
.sidebar::before {
  content: "Learna";

  display: flex;
  align-items: center;

  height: 42px;
  padding-left: 12px;
  margin-bottom: 32px;

  color: #5268e8;
  font-size: 25px;
  font-weight: 750;
  letter-spacing: -1px;
}

/* 导航区域 */
.sidebar-nav {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 6px;
}

/* 导航链接 */
.sidebar-nav a {
  display: flex;
  align-items: center;

  min-height: 44px;
  padding: 0 14px;

  border-radius: 9px;

  color: #697386;
  font-size: 14px;
  font-weight: 500;
  text-decoration: none;

  transition:
    background-color 0.2s ease,
    color 0.2s ease;
}

/* 鼠标悬停 */
.sidebar-nav a:hover {
  color: #5268e8;
  background: #f3f5ff;
}

/* 当前路由 */
.sidebar-nav a.router-link-active {
  color: #5268e8;
  background: #eef0ff;
  font-weight: 600;
}

/* 键盘焦点 */
.sidebar-nav a:focus-visible,
.sidebar-auth a:focus-visible,
.logout-btn:focus-visible {
  outline: 2px solid #647bff;
  outline-offset: 3px;
}

/* ==============================
   Sidebar Authentication
============================== */

.sidebar-auth {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;

  min-height: 56px;
  padding: 12px 8px 0;
  margin-top: 20px;

  border-top: 1px solid #eaecf2;
}

.user-name {
  overflow: hidden;

  color: #3d4556;
  font-size: 13px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 登录链接 */
.sidebar-auth a {
  color: #5268e8;
  font-size: 14px;
  font-weight: 600;
  text-decoration: none;
}

.sidebar-auth a:hover {
  color: #394fc7;
}

/* 退出按钮 */
.logout-btn {
  flex-shrink: 0;

  padding: 6px 10px;

  border: none;
  border-radius: 6px;

  background: transparent;
  color: #9299a8;

  font-size: 12px;
  cursor: pointer;

  transition:
    color 0.2s ease,
    background-color 0.2s ease;
}

.logout-btn:hover {
  color: #dc5151;
  background: #fff1f1;
}

/* ==============================
   Main
============================== */

.main {
  display: flex;
  flex: 1;
  flex-direction: column;

  min-width: 0;
  min-height: 100vh;
}

/* ==============================
   Topbar
============================== */

.topbar {
  display: flex;
  flex-shrink: 0;
  align-items: center;

  min-height: 64px;
  padding: 0 28px;

  border-bottom: 1px solid #eaecf2;
  background: #ffffff;

  color: #303849;
  font-size: 15px;
  font-weight: 600;
}

/* ==============================
   Content
============================== */

.content {
  flex: 1;

  min-width: 0;
  min-height: 0;
  padding: 28px;

  overflow-y: auto;
}

/* ==============================
   Blank Layout
   登录等独立页面
============================== */

.layout--blank {
  background: #f5f6fa;
}

.layout--blank .main {
  min-height: 100vh;
}

.layout--blank .content {
  padding: 0;
  overflow: visible;
}

/* ==============================
   Responsive
============================== */

@media (max-width: 768px) {
  .sidebar {
    width: 200px;
    padding-right: 12px;
    padding-left: 12px;
  }

  .content {
    padding: 20px;
  }

  .topbar {
    padding: 0 20px;
  }
}

@media (max-width: 600px) {
  .layout {
    flex-direction: column;
  }

  .sidebar {
    flex-direction: row;
    align-items: center;

    width: 100%;
    padding: 10px 12px;

    border-right: none;
    border-bottom: 1px solid #eaecf2;
  }

  .sidebar::before {
    flex-shrink: 0;

    height: auto;
    padding: 0;
    margin: 0 12px 0 0;

    font-size: 19px;
  }

  .sidebar-nav {
    flex-direction: row;
    gap: 4px;

    min-width: 0;
    overflow-x: auto;
    scrollbar-width: none;
  }

  .sidebar-nav::-webkit-scrollbar {
    display: none;
  }

  .sidebar-nav a {
    flex-shrink: 0;
    min-height: 38px;
    padding: 0 10px;

    font-size: 12px;
  }

  .sidebar-auth {
    flex-shrink: 0;
    min-height: auto;
    padding: 0 0 0 10px;
    margin: 0 0 0 8px;

    border-top: none;
    border-left: 1px solid #eaecf2;
  }

  .user-name {
    max-width: 75px;
  }

  .main {
    min-height: 0;
  }

  .content {
    padding: 16px;
  }

  .topbar {
    min-height: 52px;
    padding: 0 16px;
  }
}
</style>
