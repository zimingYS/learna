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
    component: () => import("../views/DashboardView.vue"),
  },
  {
    path: "/projects",
    name: "projects",
    component: () => import("../views/ProjectsView.vue"),
  },
  {
    path: "/projects/:projectId",
    name: "project-detail",
    component: () => import("../views/ProjectDetailView.vue"),
  },
  {
    path: "/records",
    name: "records",
    component: () => import("../views/RecordsView.vue"),
  },
  {
    path: "/roadmaps",
    name: "roadmaps",
    component: () => import("../views/RoadmapsView.vue"),
  },
  {
    path: "/plans",
    name: "plans",
    component: () => import("../views/PlansView.vue"),
  },
  {
    path: "/reports",
    name: "reports",
    component: () => import("../views/ReportsView.vue"),
  },
  {
    path: "/settings",
    name: "settings",
    component: () => import("../views/SettingsView.vue"),
  },
  {
    path: "/:pathMatch(.*)*",
    name: "not-found",
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
