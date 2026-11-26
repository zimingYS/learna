<template>
  <main class="login-page">
    <section class="auth-card">
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
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: #f7f8fc;
  box-sizing: border-box;
}

.auth-card {
  width: 100%;
  max-width: 360px;
  padding: 32px;
  box-sizing: border-box;
  background: #ffffff;
  border: 1px solid #e8eaf0;
  border-radius: 14px;
  box-shadow: 0 8px 30px rgb(0 0 0 / 5%);
}

.auth-header {
  margin-bottom: 28px;
  text-align: center;
}

.auth-title {
  margin: 0 0 10px;
  font-size: 24px;
  font-weight: 600;
  color: #202534;
}

.auth-description {
  margin: 0;
  font-size: 13px;
  color: #9298a8;
}

.auth-form {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-group label {
  font-size: 13px;
  font-weight: 500;
  color: #454b5c;
}

.form-group input {
  width: 100%;
  height: 42px;
  padding: 0 12px;
  box-sizing: border-box;
  border: 1px solid #e1e4eb;
  border-radius: 7px;
  outline: none;
  font-size: 13px;
  transition: border-color 0.2s;
}

.form-group input:focus {
  border-color: #647bff;
}

.submit-btn {
  height: 44px;
  margin-top: 4px;
  border: none;
  border-radius: 7px;
  background: #647bff;
  color: #ffffff;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.submit-btn:hover {
  background: #5269eb;
}

.error-msg {
  margin: 0;
  color: #dc4545;
  font-size: 12px;
  line-height: 1.5;
}

.auth-footer {
  display: flex;
  justify-content: center;
  gap: 4px;
  margin-top: 24px;
  font-size: 13px;
  color: #9298a8;
}

.toggle-btn {
  padding: 0;
  border: none;
  background: none;
  color: #647bff;
  font-size: 13px;
  cursor: pointer;
}

.toggle-btn:hover {
  color: #5269eb;
}
</style>
