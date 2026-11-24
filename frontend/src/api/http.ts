import axios, {type AxiosRequestConfig} from "axios";

// 后端统一响应信封
export interface ApiResponse<T> {
  code: string;
  message: string;
  data: T | null;
  request_id: string;
}

// 统一 API 错误对象
interface ApiErrorOptions {
  code?: string;
  request_id?: string;
  status?: number;
}

export class ApiError extends Error {
  readonly code?: string;
  readonly request_id?: string;
  readonly status?: number;

  constructor(message: string, options: ApiErrorOptions = {}) {
    super(message);

    this.name = "ApiError";
    this.code = options.code;
    this.request_id = options.request_id;
    this.status = options.status;
  }
}

export const TOKEN_KEY = "learna_token";
export const USER_KEY = "learna_user";

// Axios 实例
const http = axios.create({
  baseURL: "/api",
  timeout: 12_000,
});

// 请求拦截器, 自动注入JWT
http.interceptors.request.use((config) => {
  const token = localStorage.getItem(TOKEN_KEY);

  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }

  return config;
})

// 请求拦截器，统一处理认证失败
http.interceptors.response.use(
  (response) => response,

  (error: unknown) => {
    if (axios.isAxiosError(error) && error.response?.status === 401) {
      error.config?.headers?.get("Authorization");
      const requestUrl = error.config?.url ?? "";

      // 登录和注册接口不参与 Token 过期跳转
      const isAuthRequest = requestUrl.endsWith("/auth/login") || requestUrl.endsWith("/auth/register");

      if (!isAuthRequest) {
        // 清理认证缓存
        localStorage.removeItem(TOKEN_KEY);
        localStorage.removeItem(USER_KEY);

        // 整页跳转，让 Pinia 重新初始化
        window.location.href = "/login";
      }
    }

    // 向上抛出错误
    return Promise.reject(error);
  }
)

// 判断未知数据是否为普通对象
function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

// 从未知对象中安全读取字符串字段
function getStringField(
  record: Record<string, unknown> | undefined,
  key: string,
): string | undefined {
  const value = record?.[key];

  return typeof value === "string" ? value : undefined;
}

// 统一请求出口
export async function request<T>(
  config: AxiosRequestConfig,
): Promise<T> {
  try {
    const response = await http.request<ApiResponse<T>>(config);
    const envelope = response.data;

    if (envelope.code !== "200") {
      throw new ApiError(envelope.message?.trim() || "请求失败", {
        code: envelope.code,
        request_id: envelope.request_id,
        status: response.status,
      });
    }

    return envelope.data as T;
  } catch (error: unknown) {
    if (error instanceof ApiError) {
      throw error;
    }

    // 处理 Axios HTTP 错误和网络错误
    if (axios.isAxiosError(error)) {
      const status = error.response?.status;

      const body = isRecord(error.response?.data)
        ? error.response?.data
        : undefined;

      const bodyMessage = getStringField(body, "message");

      // HTTP 响应时优先使用状态描述
      const fallbackMessage =
        status !== undefined
          ? error.response?.statusText?.trim() || `HTTP ${status}`
          : error.message || "网络请求失败";

      throw new ApiError(bodyMessage?.trim() ? bodyMessage : fallbackMessage, {
        code: getStringField(body, "code"),
        request_id: getStringField(body, "request_id"),
        status,
      });
    }

    if (error instanceof Error) {
      throw new ApiError(error.message);
    }

    throw new ApiError("未知请求错误");
  }
}

export default http;
