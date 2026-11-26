import {AxiosError, type AxiosResponse} from "axios"
import {describe, expect, it, vi} from "vitest"
import http, {type ApiResponse, ApiError, request} from "../src/api/http"

describe("request 统一响应处理", () => {
  it("code 为 200 时返回信封中的 data", async () => {
    vi.spyOn(http, "request").mockResolvedValue({
      data: {code: "200", message: "ok", data: {id: "1"}, request_id: "req-1"},
      status: 200,
    } as AxiosResponse<ApiResponse<{id: string}>>)

    await expect(request<{id: string}>({url: "/ping", method: "get"})).resolves.toEqual({
      id: "1",
    })
  })

  it("业务错误码转换为 ApiError 并保留错误码与 request_id", async () => {
    vi.spyOn(http, "request").mockResolvedValue({
      data: {
        code: "40001",
        message: "参数错误",
        data: null,
        request_id: "req-2",
      },
      status: 200,
    } as AxiosResponse<ApiResponse<null>>)

    await expect(request({url: "/ping", method: "get"})).rejects.toMatchObject({
      name: "ApiError",
      message: "参数错误",
      code: "40001",
      request_id: "req-2",
      status: 200,
    })
  })

  it("HTTP 错误优先使用后端消息，缺失时回退到状态码", async () => {
    const withBody = new AxiosError(
      "Request failed with status code 400",
      "ERR_BAD_REQUEST",
      undefined,
      undefined,
      {
        status: 400,
        statusText: "Bad Request",
        data: {code: "40002", message: "邮箱已存在", request_id: "req-3"},
      } as AxiosResponse,
    )
    vi.spyOn(http, "request").mockRejectedValue(withBody)

    await expect(request({url: "/register", method: "post"})).rejects.toMatchObject({
      message: "邮箱已存在",
      code: "40002",
      request_id: "req-3",
      status: 400,
    })

    const withoutBody = new AxiosError(
      "Request failed with status code 502",
      "ERR_BAD_RESPONSE",
      undefined,
      undefined,
      {status: 502, statusText: "", data: ""} as AxiosResponse,
    )
    vi.spyOn(http, "request").mockRejectedValue(withoutBody)

    await expect(request({url: "/ping", method: "get"})).rejects.toMatchObject({
      message: "HTTP 502",
      status: 502,
    })
  })

  it("非 Axios 异常统一包装为 ApiError", async () => {
    vi.spyOn(http, "request").mockRejectedValue(new Error("boom"))

    const error = await request({url: "/ping", method: "get"}).catch(
      (reason: unknown) => reason,
    )

    expect(error).toBeInstanceOf(ApiError)
    expect(error).toMatchObject({message: "boom"})
  })
})
