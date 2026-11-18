from __future__ import annotations

from fastapi import APIRouter
from fastapi.responses import JSONResponse

from app.schemas.base import ApiResponse

router = APIRouter()


@router.post("/documents/parse")
async def parse_document() -> JSONResponse:
    return JSONResponse(
        status_code=501,
        content=ApiResponse.error(
            "NOT_IMPLEMENTED", "POST /internal/documents/parse not implemented"
        ).model_dump(),
    )
