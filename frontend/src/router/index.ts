import {
  createRouter,
  createWebHistory,
  type RouteRecordRaw,
} from "vue-router";

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

export default router;
