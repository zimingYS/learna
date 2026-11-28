<template>
  <main class="login-page">
    <section class="auth-card">
      <div class="auth-brand">
        <span class="brand-mark">
          <svg aria-hidden="true" fill="none" stroke="currentColor" stroke-linecap="round"
               stroke-linejoin="round" stroke-width="1.8" viewBox="0 0 24 24">
            <path d="M3 8.5 12 4l9 4.5-9 4.5-9-4.5Z"/>
            <path d="M7 11v4.6c0 1.3 2.2 2.4 5 2.4s5-1.1 5-2.4V11"/>
          </svg>
        </span>
        <span class="brand-name">Learna</span>
      </div>

      <header class="auth-header">
        <h2 class="auth-title">
          {{ isRegister ? "创建账号" : "欢迎回来" }}
        </h2>

        <p class="auth-description">
          {{ isRegister ? "注册 Learna，开启学习之旅" : "登录 Learna，继续你的学习之旅" }}
        </p>
      </header>

      <form class="auth-form" @submit.prevent="handleSubmit">
        <!-- 显示名称 仅在注册时展示 -->
        <div v-if="isRegister" class="form-group">
          <label for="display_name">显示名称</label>
          <input id="display_name" v-model="form.display_name.value" type="text" placeholder="请输入显示名称"
                 autocomplete="nickname" required/>
        </div>

        <!-- 邮箱 -->
        <div class="form-group">
          <label for="email">邮箱</label>
          <input id="email" v-model="form.email.value" type="email" placeholder="请输入邮箱" autocomplete="email"
                 required/>
        </div>

        <!-- 密码 -->
        <div class="form-group">
          <label for="password">密码</label>
          <input id="password" v-model="form.password.value" type="password" placeholder="请输入密码"
                 :autocomplete="isRegister? 'new-password' : 'current-password'" required/>
        </div>

        <!-- 错误提示 -->
        <p v-if="errorMessage" class="error-msg" role="alert">
          {{ errorMessage }}
        </p>

        <!-- 提交 -->
        <button type="submit" class="submit-btn">
          {{ isRegister ? "注册" : "登录" }}
        </button>
      </form>

      <!-- 模式切换 -->
      <footer class="auth-footer">
        <span>
          {{ isRegister ? "已有账号" : "没有账号?" }}
        </span>

        <button type="button" class="toggle-btn" @click="toggleMode">
          {{ isRegister ? "登录" : "注册" }}
        </button>
      </footer>
    </section>
  </main>
</template>

<script setup lang="ts">
import {onMounted, ref} from "vue";
import {useAuthStore} from "../stores/auth.ts";
import {ApiError} from "../api/http.ts";
import {useRouter} from "vue-router";

const router = useRouter();
const authStore = useAuthStore();

const isRegister = ref(false);

const form = {
  email: ref(""),
  password: ref(""),
  display_name: ref(""),
};

const errorMessage = ref("");

/// 提交登录/注册
async function handleSubmit(): Promise<void> {
  errorMessage.value = "";

  try {
    if (isRegister.value) {
      await authStore.register({
        email: form.email.value,
        password: form.password.value,
        display_name: form.display_name.value,
      });
    } else {
      await authStore.login({
        email: form.email.value,
        password: form.password.value,
      });
    }

    await router.push("/dashboard");
  } catch (e: unknown) {
    errorMessage.value = e instanceof ApiError ? e.message : "请求失败";
  }
}

/// 切换注册/登录
function toggleMode(): void {
  isRegister.value = !isRegister.value;

  // 清除错误信息
  errorMessage.value = "";
}

/// 已登录用户自动跳转
onMounted(() => {
  if (authStore.isLoggedIn) {
    void router.push("/dashboard")
  }
})
</script>

<style scoped>
.login-page {
  display: flex;
  align-items: center;
  justify-content: center;

  min-height: 100vh;
  padding: 24px;

  background: var(--canvas-grad);
}

.auth-card {
  width: 100%;
  max-width: 384px;
  padding: 34px 32px 30px;

  border: 1px solid var(--line);
  border-radius: 22px;

  background: var(--surface);
  box-shadow: var(--shadow-md);
}

.auth-brand {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 9px;

  margin-bottom: 22px;
}

.brand-mark {
  display: flex;
  align-items: center;
  justify-content: center;

  width: 30px;
  height: 30px;

  border-radius: 9px;

  background: linear-gradient(140deg, #6d6df6, var(--brand));
  box-shadow: 0 6px 14px -6px rgb(79 70 229 / 55%);
  color: #fff;
}

.brand-mark svg {
  width: 18px;
  height: 18px;
}

.brand-name {
  color: var(--ink-1);
  font-size: 18px;
  font-weight: 750;
  letter-spacing: -0.4px;
}

.auth-header {
  margin-bottom: 26px;
  text-align: center;
}

.auth-title {
  margin: 0 0 8px;

  color: var(--ink-1);
  font-size: 21px;
  font-weight: 700;
  letter-spacing: -0.4px;
}

.auth-description {
  margin: 0;

  color: var(--ink-3);
  font-size: 12.5px;
}

.auth-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.form-group label {
  color: var(--ink-2);
  font-size: 12.5px;
  font-weight: 600;
}

.form-group input {
  width: 100%;
  height: 44px;
  padding: 0 14px;

  border: 1px solid var(--line);
  border-radius: var(--radius-sm);
  outline: none;

  background: var(--surface-muted);
  color: var(--ink-1);

  font-size: 13.5px;

  transition: border-color 0.18s var(--ease), background-color 0.18s var(--ease),
  box-shadow 0.18s var(--ease);
}

.form-group input::placeholder {
  color: var(--ink-4);
}

.form-group input:focus {
  border-color: var(--brand);
  background: var(--surface);
  box-shadow: 0 0 0 3px var(--brand-ring);
}

.submit-btn {
  height: 44px;
  margin-top: 6px;

  border: none;
  border-radius: var(--radius-pill);

  background: linear-gradient(135deg, #6d6df6, var(--brand));
  box-shadow: 0 10px 22px -12px rgb(79 70 229 / 85%);
  color: #fff;

  font-size: 14px;
  font-weight: 600;
  cursor: pointer;

  transition: transform 0.18s var(--ease), box-shadow 0.18s var(--ease);
}

.submit-btn:hover {
  box-shadow: 0 14px 26px -12px rgb(79 70 229 / 90%);
  transform: translateY(-1px);
}

.error-msg {
  padding: 9px 12px;
  margin: 0;

  border: 1px solid #f7dada;
  border-radius: var(--radius-sm);

  background: var(--rose-soft);
  color: #bf3a40;

  font-size: 12px;
  line-height: 1.5;
}

.auth-footer {
  display: flex;
  justify-content: center;
  gap: 4px;

  margin-top: 22px;

  color: var(--ink-3);
  font-size: 12.5px;
}

.toggle-btn {
  padding: 0;

  border: none;

  background: none;
  color: var(--brand);

  font-size: 12.5px;
  font-weight: 600;
  cursor: pointer;
}

.toggle-btn:hover {
  color: var(--brand-strong);
}
</style>
