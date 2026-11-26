import {
  type AuthResponse,
  type AuthUser,
  login as loginApi,
  type LoginRequest,
  register as registerApi,
  type RegisterRequest
} from "../api/auth.ts";
import {defineStore} from "pinia";
import {computed, ref} from "vue";
import {TOKEN_KEY, USER_KEY} from "../api/http.ts";

/// 校验恢复出来的用户数据结构
function isAuthUser(value: unknown): value is AuthUser {
  if (typeof value !== "object" || value === null) {
    return false;
  }

  const user = value as Record<string, unknown>;

  return (
    typeof user.id === "string" &&
    typeof user.email === "string" &&
    typeof user.display_name === "string"
  );
}

/// 从 localStorage 恢复认证状态
function restoreAuth(): {
  token: string | null;
  user: AuthUser | null;
} {
  const storedToken = localStorage.getItem(TOKEN_KEY);
  const storedUser = localStorage.getItem(USER_KEY);

  // Token 或用户信息缺失，统一视为未登录
  if (!storedToken || !storedUser) {
    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(USER_KEY);

    return {
      token: null,
      user: null,
    };
  }

  try {
    const parsed: unknown = JSON.parse(storedUser);

    if (!isAuthUser(parsed)) {
      throw new Error("无效的存储用户数据");
    }

    return {
      token: storedToken,
      user: parsed,
    };
  } catch {
    // JSON 解析失败或用户结构错误，清理损坏的数据
    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(USER_KEY);

    return {
      token: null,
      user: null,
    };
  }
}

export const useAuthStore = defineStore("auth", () => {
  const restored = restoreAuth();

  const token = ref<string | null>(restored.token);
  const user = ref<AuthUser | null>(restored.user);

  const isLoggedIn = computed(() => token.value !== null);

  /// 应用认证结果并持久化
  function applyAuth(res: AuthResponse): void {
    token.value = res.token;
    user.value = res.user;

    localStorage.setItem(TOKEN_KEY, res.token);
    localStorage.setItem(USER_KEY, JSON.stringify(res.user));
  }

  /// 用户登录
  async function login(req: LoginRequest): Promise<AuthResponse> {
    const res = await loginApi(req);

    applyAuth(res);

    return res;
  }

  /// 用户注册
  async function register(req: RegisterRequest): Promise<AuthResponse> {
    const res = await registerApi(req);

    applyAuth(res);

    return res;
  }

  /// 用户退出登录
  function logout(): void {
    token.value = null;
    user.value = null;

    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(USER_KEY);
  }

  return {
    token,
    user,

    isLoggedIn,

    login,
    register,
    logout,
  };
})
