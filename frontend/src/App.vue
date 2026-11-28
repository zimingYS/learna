<script setup lang="ts">
import {computed, onBeforeUnmount, onMounted, ref} from "vue";
import {RouterLink, RouterView, useRoute, useRouter} from "vue-router";
import {useAuthStore} from "./stores/auth.ts";

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
    to: "/contents",
    label: "学习内容",
    paths: ["M12 3 3 8l9 5 9-5-9-5Z", "M3 14l9 5 9-5"],
  },
  {
    to: "/roadmaps",
    label: "规划工作台",
    paths: ["m9 4 6 2 6-2v12l-6 2-6-2-6 2V6Z", "M9 4v12M15 6v12"],
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
    to: "/plans",
    label: "学习计划",
    paths: [
      "M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v13a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2Z",
      "M8 3v4M16 3v4M4 10h16",
    ],
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

const pageTitle = computed(() => (route.meta.title as string | undefined) ?? "工作台");

const displayName = computed(() => authStore.user?.display_name ?? "");

const initial = computed(() => displayName.value.trim().slice(0, 1).toUpperCase() || "L");

const menuOpen = ref(false);
const accountRef = ref<HTMLElement | null>(null);

function handleDocumentClick(event: MouseEvent): void {
  if (accountRef.value && !accountRef.value.contains(event.target as Node)) {
    menuOpen.value = false;
  }
}

onMounted(() => document.addEventListener("click", handleDocumentClick));
onBeforeUnmount(() => document.removeEventListener("click", handleDocumentClick));

async function handleLogout(): Promise<void> {
  menuOpen.value = false;
  authStore.logout();
  await router.push("/login");
}
</script>

<template>
  <div class="layout" :class="{ 'layout--blank': route.meta.blank }">
    <template v-if="!route.meta.blank">
      <aside class="sidebar">
        <RouterLink class="brand" to="/dashboard">
          <span class="brand-mark">
            <svg aria-hidden="true" fill="none" stroke="currentColor" stroke-linecap="round"
                 stroke-linejoin="round" stroke-width="1.8" viewBox="0 0 24 24">
              <path d="M3 8.5 12 4l9 4.5-9 4.5-9-4.5Z"/>
              <path d="M7 11v4.6c0 1.3 2.2 2.4 5 2.4s5-1.1 5-2.4V11"/>
            </svg>
          </span>
          <span class="brand-name">Learna</span>
        </RouterLink>

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

        <div class="sidebar-foot">
          <div class="quote-card">
            <svg aria-hidden="true" class="quote-icon" fill="none" stroke="currentColor"
                 stroke-linecap="round" stroke-linejoin="round" stroke-width="1.6"
                 viewBox="0 0 24 24">
              <circle cx="12" cy="12" r="4"/>
              <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/>
            </svg>
            <p>持续学习，<br>遇见更好的自己</p>
          </div>

          <div v-if="authStore.isLoggedIn" class="account-chip">
            <span class="chip-avatar">{{ initial }}</span>
            <span class="chip-name">{{ displayName }}</span>
          </div>

          <RouterLink v-else class="chip-login" to="/login">登录</RouterLink>
        </div>
      </aside>
    </template>

    <div class="main">
      <template v-if="!route.meta.blank">
        <header class="topbar">
          <p class="crumb">
            <span class="crumb-root">工作台</span>
            <span aria-hidden="true" class="crumb-sep">/</span>
            <strong>{{ pageTitle }}</strong>
          </p>

          <div class="top-actions">
            <div ref="accountRef" class="account">
              <button
                :aria-expanded="menuOpen"
                aria-haspopup="menu"
                class="avatar-btn"
                type="button"
                @click="menuOpen = !menuOpen"
              >
                <span class="avatar">{{ initial }}</span>
                <svg aria-hidden="true" class="chevron" fill="none" stroke="currentColor"
                     stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                     viewBox="0 0 24 24">
                  <path d="m6 9 6 6 6-6"/>
                </svg>
              </button>

              <Transition name="menu">
                <div v-if="menuOpen" class="account-menu" role="menu">
                  <p class="menu-name">{{ displayName || "未登录" }}</p>
                  <p v-if="authStore.user?.email" class="menu-mail">{{ authStore.user.email }}</p>

                  <button class="menu-item" role="menuitem" type="button" @click="handleLogout">
                    退出登录
                  </button>
                </div>
              </Transition>
            </div>
          </div>
        </header>
      </template>

      <main class="content" :class="{ 'content--blank': route.meta.blank }">
        <RouterView/>
      </main>
    </div>
  </div>
</template>

<style scoped>
.layout {
  display: flex;
  width: 100%;
  min-height: 100vh;
  background: var(--canvas);
}

/* ==============================
   侧栏
============================== */

.sidebar {
  position: sticky;
  top: 0;

  display: flex;
  flex-shrink: 0;
  flex-direction: column;

  width: 236px;
  height: 100vh;
  padding: 22px 14px 16px;

  border-right: 1px solid var(--line);
  background: var(--surface);
  overflow-y: auto;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;

  height: 40px;
  padding: 0 8px;
  margin-bottom: 22px;

  color: var(--ink-1);
  text-decoration: none;
}

.brand-mark {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;

  width: 32px;
  height: 32px;

  border-radius: 10px;

  background: linear-gradient(140deg, #6d6df6, var(--brand));
  box-shadow: 0 6px 14px -6px rgb(79 70 229 / 55%);
  color: #fff;
}

.brand-mark svg {
  width: 19px;
  height: 19px;
}

.brand-name {
  font-size: 19px;
  font-weight: 750;
  letter-spacing: -0.4px;
}

.sidebar-nav {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 3px;
}

.sidebar-nav a {
  display: flex;
  align-items: center;
  gap: 11px;

  min-height: 42px;
  padding: 0 12px;

  border-radius: var(--radius-sm);

  color: var(--ink-2);
  font-size: 13.5px;
  font-weight: 500;
  text-decoration: none;

  transition: background-color 0.18s var(--ease), color 0.18s var(--ease);
}

.nav-icon {
  flex-shrink: 0;

  width: 18px;
  height: 18px;

  opacity: 0.78;
}

.sidebar-nav a:hover {
  background: var(--surface-muted);
  color: var(--brand);
}

.sidebar-nav a.router-link-active {
  background: var(--brand-soft);
  color: var(--brand);
  font-weight: 600;
}

.sidebar-nav a.router-link-active .nav-icon {
  opacity: 1;
}

/* ==============================
   侧栏底部
============================== */

.sidebar-foot {
  display: flex;
  flex-direction: column;
  gap: 12px;

  padding-top: 14px;
}

.quote-card {
  padding: 14px;
  border-radius: var(--radius-md);
  background: linear-gradient(150deg, #fafaff, #f2f0ff);
  color: var(--ink-3);
}

.quote-icon {
  width: 18px;
  height: 18px;
  margin-bottom: 8px;
  color: #f0b429;
}

.quote-card p {
  margin: 0;
  font-size: 12px;
  line-height: 1.7;
}

.account-chip {
  display: flex;
  align-items: center;
  gap: 9px;

  padding: 6px 8px;

  border-radius: var(--radius-sm);

  background: var(--surface-muted);
}

.chip-avatar {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;

  width: 26px;
  height: 26px;

  border-radius: 50%;

  background: var(--brand-soft);
  color: var(--brand);

  font-size: 12px;
  font-weight: 700;
}

.chip-name {
  overflow: hidden;

  color: var(--ink-2);
  font-size: 12.5px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chip-login {
  padding: 8px 10px;

  border-radius: var(--radius-sm);

  background: var(--brand-soft);
  color: var(--brand);

  font-size: 13px;
  font-weight: 600;
  text-decoration: none;
}

/* ==============================
   主区
============================== */

.main {
  display: flex;
  flex: 1;
  flex-direction: column;

  min-width: 0;
  min-height: 100vh;

  background: var(--canvas-grad);
}

.topbar {
  position: sticky;
  z-index: 20;
  top: 0;

  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: space-between;
  gap: 20px;

  min-height: 62px;
  padding: 14px 36px 12px;

  background: rgb(249 250 255 / 72%);
  backdrop-filter: blur(12px);
}

.crumb {
  display: flex;
  align-items: center;
  gap: 8px;

  margin: 0;

  color: var(--ink-3);
  font-size: 13px;
}

.crumb-sep {
  color: var(--ink-4);
}

.crumb strong {
  color: var(--ink-1);
  font-weight: 600;
}

.top-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.account {
  position: relative;
}

.avatar-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;

  height: 36px;
  padding: 0 10px 0 4px;

  border: 1px solid var(--line);
  border-radius: var(--radius-pill);

  background: #fff;
  cursor: pointer;

  transition: border-color 0.18s var(--ease), box-shadow 0.18s var(--ease);
}

.avatar-btn:hover {
  border-color: var(--line-strong);
  box-shadow: var(--shadow-xs);
}

.avatar {
  display: flex;
  align-items: center;
  justify-content: center;

  width: 28px;
  height: 28px;

  border-radius: 50%;

  background: linear-gradient(140deg, #7d7cf8, var(--brand));
  color: #fff;

  font-size: 12px;
  font-weight: 700;
}

.chevron {
  width: 13px;
  height: 13px;
  color: var(--ink-4);
}

.account-menu {
  position: absolute;
  z-index: 30;
  top: calc(100% + 8px);
  right: 0;

  width: 208px;
  padding: 12px;

  border: 1px solid var(--line);
  border-radius: var(--radius-md);

  background: #fff;
  box-shadow: var(--shadow-md);
}

.menu-name {
  margin: 0;
  color: var(--ink-1);
  font-size: 13px;
  font-weight: 650;
}

.menu-mail {
  overflow: hidden;

  margin: 3px 0 10px;

  color: var(--ink-4);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.menu-item {
  width: 100%;
  padding: 8px 10px;

  border: none;
  border-radius: 8px;

  background: transparent;
  color: var(--rose);

  font-size: 13px;
  font-weight: 600;
  text-align: left;
  cursor: pointer;

  transition: background-color 0.18s var(--ease);
}

.menu-item:hover {
  background: var(--rose-soft);
}

.menu-enter-active,
.menu-leave-active {
  transition: opacity 0.14s var(--ease), transform 0.14s var(--ease);
}

.menu-enter-from,
.menu-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

.content {
  flex: 1;

  min-width: 0;
  min-height: 0;
  padding: 26px 36px 52px;
}

.content--blank {
  min-height: 100vh;
  padding: 0;
}

.layout--blank .main {
  min-height: 100vh;
}

/* ==============================
   响应式
============================== */

@media (max-width: 1024px) {
  .sidebar {
    width: 208px;
  }

  .topbar {
    padding: 12px 20px 0;
  }

  .content {
    padding: 22px 20px 40px;
  }
}

@media (max-width: 760px) {
  .layout {
    flex-direction: column;
  }

  .sidebar {
    position: static;

    flex-direction: row;
    align-items: center;
    gap: 8px;

    width: 100%;
    height: auto;
    padding: 10px 12px;

    border-right: none;
    border-bottom: 1px solid var(--line);
    overflow: visible;
  }

  .brand {
    height: auto;
    padding: 0;
    margin: 0;
  }

  .brand-name,
  .sidebar-foot {
    display: none;
  }

  .sidebar-nav {
    flex-direction: row;
    gap: 2px;

    min-width: 0;
    overflow-x: auto;
    scrollbar-width: none;
  }

  .sidebar-nav::-webkit-scrollbar {
    display: none;
  }

  .sidebar-nav a {
    flex-shrink: 0;
    min-height: 36px;
    padding: 0 10px;

    font-size: 12.5px;
  }

  .topbar {
    min-height: 52px;
    padding: 10px 16px 0;
  }

  .content {
    padding: 16px 16px 32px;
  }

}
</style>
