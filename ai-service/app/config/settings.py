from __future__ import annotations

from functools import lru_cache

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(
        # 根 .env 兜底(README 让在根目录 cp .env.example), ai-service/.env 可覆盖
        # 容器内 ../.env 不存在, 缺失文件会被静默跳过, 密钥只走环境变量
        env_file=("../.env", ".env"),
        env_file_encoding="utf-8",
        env_ignore_empty=True,
        extra="ignore",
    )

    ai_service_token: str | None = None

    @property
    def internal_token(self) -> str:
        """内部凭据; 去首尾空白(密钥文件常带换行), 空串等价于未配置"""
        return (self.ai_service_token or "").strip()


@lru_cache(maxsize=1)
def get_settings() -> Settings:
    return Settings()
