<script setup lang="ts">
import {RouterLink, RouterView, useRoute, useRouter} from "vue-router";
import {useAppStore} from "./stores/app"
import {useAuthStore} from "./stores/auth.ts";

const appStore = useAppStore();
const authStore = useAuthStore();

const route = useRoute();
const router = useRouter();

// 侧栏导航：图标路径与 docs/产品需求/设计稿/UI概念图-首页仪表盘.png 保持同一线性风格
const navItems: { to: string; label: string; paths: string[] }[] = [
  { to: "/dashboard", label: "首页", paths: ["m3 10.5 9-7 9 7", "M5 9.5V20h14V9.5"] },
  {
    to: "/projects",
    label: "学习项目",
    paths: [
      "M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z",
    ],
  },
  {
    to: "/plans",
    label: "学习计划",
    paths: [
      "M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v13a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2Z",
      "M8 3v4M16 3v4M4 10h16",
    ],
  },
  {
    to: "/records",
    label: "学习记录",
    paths: [
      "M4 19.5A2.5 2.5 0 0 1 6.5 17H20",
      "M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2Z",
    ],
  },
  {
    to: "/roadmaps",
    label: "路线图",
    paths: ["m9 4 6 2 6-2v12l-6 2-6-2-6 2V6Z", "M9 4v12M15 6v12"],
  },
  { to: "/reports", label: "报告", paths: ["M6 20v-6M12 20V6M18 20v-9", "M3 20h18"] },
  {
    to: "/settings",
    label: "设置",
    paths: [
      "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z",
      "M10.3 3h3.4l.4 2.2 2 1.2 2.1-.8 1.7 2.9-1.6 1.6v2.4l1.6 1.6-1.7 2.9-2.1-.8-2 1.2-.4 2.2h-3.4l-.4-2.2-2-1.2-2.1.8-1.7-2.9 1.6-1.6v-2.4L3.1 8.5l1.7-2.9 2.1.8 2-1.2Z",
    ],
  },
];

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
          <RouterLink v-for="item in navItems" :key="item.to" :to="item.to">
            <svg
              aria-hidden="true"
              class="nav-icon"
              fill="none"
              stroke="currentColor"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="1.7"
              viewBox="0 0 24 24"
            >
              <path v-for="path in item.paths" :key="path" :d="path"/>
            </svg>
            <span>{{ item.label }}</span>
          </RouterLink>
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
  gap: 11px;

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

.nav-icon {
  flex-shrink: 0;

  width: 18px;
  height: 18px;

  opacity: 0.85;
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
