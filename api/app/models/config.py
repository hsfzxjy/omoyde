from pathlib import Path
from pydantic import BaseSettings


class TencentCOSConfig(BaseSettings):
    bucket: str
    region: str


class TencentCloudConfig(BaseSettings):
    appId: str
    secretId: str
    secretKey: str
    cos: TencentCOSConfig


class WeixinConfig(BaseSettings):
    appId: str
    appSecret: str


class SecurityConfig(BaseSettings):
    pincode: str
    password: str
    secret: str


class SystemConfig(BaseSettings):
    domain: str

    class Config:
        extra = "ignore"


class Config(BaseSettings):
    redis_url: str = "redis://127.0.0.1:6379"
    tcloud: TencentCloudConfig
    security: SecurityConfig
    wx: WeixinConfig
    system: SystemConfig

    class Config:
        extra = "ignore"


cfg = Config.parse_file(Path(__file__).parent.parent / "config.json")
