from pathlib import Path
from pydantic import BaseSettings


class TencentCloudConfig(BaseSettings):
    appId: str
    secretId: str
    secretKey: str


class TencentCOSConfig(BaseSettings):
    bucket: str
    region: str


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
    cos: TencentCOSConfig
    security: SecurityConfig
    wx: WeixinConfig
    system: SystemConfig

    class Config:
        extra = "ignore"


cfg = Config.parse_file(Path(__file__).parent.parent / "config.json")
