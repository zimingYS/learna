import uuid
from typing import Any, Final

from pydantic import BaseModel, ConfigDict, Field

SCHEMA_VERSION: Final[str] = "v1"
SUCCESS_CODE: Final[str] = "200"


class ApiResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    code: str
    message: str
    data: Any | None = None
    request_id: str = Field(default_factory=lambda: uuid.uuid4().hex)
    schema_version: str = SCHEMA_VERSION

    # 构造器
    @classmethod
    def ok(
        cls,
        data: Any = None,
        *,
        message: str = "success",
        request_id: str | None = None,
    ) -> "ApiResponse":
        return cls(
            code=SUCCESS_CODE,
            message=message,
            data=data,
            request_id=request_id if request_id is not None else uuid.uuid4().hex,
        )

    @classmethod
    def error(
        cls,
        code: str,
        message: str,
        *,
        request_id: str | None = None,
    ) -> "ApiResponse":
        return cls(
            code=code,
            message=message,
            data=None,
            request_id=request_id if request_id is not None else uuid.uuid4().hex,
        )
