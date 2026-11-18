from __future__ import annotations
import logging
import secrets
from fastapi import Depends, Header, HTTPException
from app.config.settings import Settings, get_settings
from app.schemas.base import ApiResponse

logger = logging.getLogger(__name__)

INTERNAL_TOKEN_HEADER = "X-Internal-Token"
INVALID_TOKEN_CODE = "40101"


class InternalAuthError(HTTPException):
    """统一错误信封"""

    def __init__(self, message: str = "invalid internal token") -> None:
        envelope = ApiResponse.error(INVALID_TOKEN_CODE, message)
        super().__init__(status_code=401, detail=envelope.model_dump())


async def require_internal_token(
    x_internal_token: str | None = Header(default=None, alias=INTERNAL_TOKEN_HEADER),
    settings: Settings = Depends(get_settings),
) -> None:
    expected = settings.ai_service_token

    if expected is None:
        logger.warning(
            "internal auth permissive mode: AI_SERVICE_TOKEN unset, allowing request "
            "(header_present=%s); 联调前收紧为 401",
            x_internal_token is not None,
        )
        return

    if x_internal_token is None or not secrets.compare_digest(
        x_internal_token, expected
    ):
        logger.info(
            "internal auth rejected: %s missing or mismatch",
            INTERNAL_TOKEN_HEADER,
        )
        raise InternalAuthError()


def install_internal_auth_handler(app) -> None:
    from fastapi.responses import JSONResponse

    @app.exception_handler(InternalAuthError)
    async def _handler(_request, exc: InternalAuthError) -> JSONResponse:
        return JSONResponse(status_code=exc.status_code, content=exc.detail)
