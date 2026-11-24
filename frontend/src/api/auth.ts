import {request} from "./http.ts";

/// 认证用户信息
export interface AuthUser {
  id: string;
  email: string;
  display_name: string;
}

/// 登录请求
export interface LoginRequest {
  email: string;
  password: string;
}

/// 注册请求
export interface RegisterRequest {
  email: string;
  password: string;
  display_name: string;
}

/// 认证响应
export interface AuthResponse {
  token: string;
  user: AuthUser;
}

/// 用户登录
export async function login(data: LoginRequest): Promise<AuthResponse> {
  return request<AuthResponse>({
    url: "/auth/login",
    method: "post",
    data,
  });
}

/// 用户注册
export async function register(data: RegisterRequest): Promise<AuthResponse> {
  return request<AuthResponse>({
    url: "/auth/register",
    method: "post",
    data,
  });
}
