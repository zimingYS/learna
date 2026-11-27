from __future__ import annotations

import logging
import secrets

from fastapi import Depends, FastAPI, Header, HTTPException, status
from fastapi.responses import JSONResponse

from app.config.settings import Settings, get_settings
from app.schemas.base import ApiResponse

logger = logging.getLogger(__name__)

INTERNAL_TOKEN_HEADER = "X-Internal-Token"
INVALID_TOKEN_CODE = "40101"
UNCONFIGURED_CODE = "INTERNAL_AUTH_UNCONFIGURED"
UNCONFIGURED_MESSAGE = (
    "internal auth not configured: AI_SERVICE_TOKEN is unset, "
    "internal requests are refused until it is provided"
)


class InternalAuthError(HTTPException):
    """内部鉴权失败, 响应体为统一错误信封"""

    def __init__(
        self,
        code: str,
        message: str,
        status_code: int = status.HTTP_401_UNAUTHORIZED,
    ) -> None:
        envelope = ApiResponse.error(code, message)
        super().__init__(status_code=status_code, detail=envelope.model_dump())


def assert_internal_auth_configured(settings: Settings) -> None:
    """启动闸门: 未配置内部凭据直接拒绝启动, 不允许以无鉴权状态对外服务"""
    if settings.internal_token:
        return
    raise RuntimeError(f"{UNCONFIGURED_MESSAGE}(see .env.example)")


async def require_internal_token(
    x_internal_token: str | None = Header(default=None, alias=INTERNAL_TOKEN_HEADER),
    settings: Settings = Depends(get_settings),  # noqa: B008  FastAPI 依赖注入标准写法
) -> None:
    expected = settings.internal_token

    # 未配置即拒绝, 绝不降级为放行(fail-closed)
    # 配置缺失属于部署故障, 不是客户端凭证问题, 因此回 503 而非 401
    if not expected:
        logger.error(UNCONFIGURED_MESSAGE)
        raise InternalAuthError(
            UNCONFIGURED_CODE,
            UNCONFIGURED_MESSAGE,
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
        )

    # 按 bytes 比较: compare_digest 遇非 ASCII str 抛 TypeError,
    # 不先编码则非法 header 会变成 500
    if x_internal_token is None or not secrets.compare_digest(
        x_internal_token.encode("utf-8"), expected.encode("utf-8")
    ):
        logger.warning(
            "internal auth rejected: %s missing or mismatch", INTERNAL_TOKEN_HEADER
        )
        raise InternalAuthError(INVALID_TOKEN_CODE, "invalid internal token")


def install_internal_auth_handler(app: FastAPI) -> None:
    @app.exception_handler(InternalAuthError)
    async def _handler(_request, exc: InternalAuthError) -> JSONResponse:
        return JSONResponse(status_code=exc.status_code, content=exc.detail)
