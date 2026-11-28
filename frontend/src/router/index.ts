import {
  createRouter,
  createWebHistory,
  type RouteRecordRaw,
} from "vue-router";
import {TOKEN_KEY} from "../api/http";

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    redirect: "/dashboard",
  },
  {
    path: "/login",
    name: "login",
    component: () => import("../views/LoginView.vue"),
    meta: {blank: true}
  },
  {
    path: "/dashboard",
    name: "dashboard",
    meta: {title: "首页"},
    component: () => import("../views/DashboardView.vue"),
  },
  {
    path: "/projects",
    name: "projects",
    meta: {title: "学习项目"},
    component: () => import("../views/ProjectsView.vue"),
  },
  {
    path: "/projects/:projectId",
    name: "project-detail",
    meta: {title: "项目详情"},
    component: () => import("../views/ProjectDetailView.vue"),
  },
  {
    path: "/records",
    name: "records",
    meta: {title: "学习记录"},
    component: () => import("../views/RecordsView.vue"),
  },
  {
    path: "/roadmaps",
    name: "roadmaps",
    meta: {title: "规划工作台"},
    component: () => import("../views/RoadmapsView.vue"),
  },
  {
    path: "/plans",
    name: "plans",
    meta: {title: "学习计划"},
    component: () => import("../views/PlansView.vue"),
  },
  {
    path: "/reports",
    name: "reports",
    meta: {title: "报告"},
    component: () => import("../views/ReportsView.vue"),
  },
  {
    path: "/settings",
    name: "settings",
    meta: {title: "设置"},
    component: () => import("../views/SettingsView.vue"),
  },
  {
    path: "/:pathMatch(.*)*",
    name: "not-found",
    meta: {title: "页面不存在"},
    component: () => import("../views/NotFoundView.vue"),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

// 全局守卫：未登录一律进登录页
router.beforeEach((to) => {
  if (!localStorage.getItem(TOKEN_KEY) && to.name !== "login") {
    return {name: "login"};
  }
});

export default router;
