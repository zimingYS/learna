from __future__ import annotations

from fastapi import APIRouter
from fastapi.responses import JSONResponse

from app.schemas.base import ApiResponse

router = APIRouter()


@router.post("/roadmaps/generate")
async def generate_roadmap() -> JSONResponse:
    return JSONResponse(
        status_code=501,
        content=ApiResponse.error(
            "NOT_IMPLEMENTED", "POST /internal/roadmaps/generate not implemented"
        ).model_dump(),
    )
