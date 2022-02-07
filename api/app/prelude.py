import aioredis
from pydantic import BaseModel
from passlib.context import CryptContext
from fastapi import Depends, HTTPException, status, FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi_jwt_auth import AuthJWT
from fastapi_jwt_auth.exceptions import AuthJWTException
from qcloud_cos import CosConfig, CosS3Client

from app.models.config import cfg


__all__ = [
    "AuthJWT",
    "app",
    "redis",
    "cfg",
    "pwd_context",
    "BaseModel",
    "Depends",
    "HTTPException",
    "status",
    "cos_client",
]

app = FastAPI(title="FastAPI Redis Tutorial")
redis = aioredis.from_url(cfg.redis_url, decode_responses=True)
pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")
cos_config = CosConfig(
    Appid=cfg.tcloud.appId,
    Region=cfg.tcloud.cos.region,
    SecretId=cfg.tcloud.secretId,
    SecretKey=cfg.tcloud.secretKey,
    Domain=cfg.system.domain,
    Scheme="https",
)
cos_client = CosS3Client(cos_config)


app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.exception_handler(AuthJWTException)
def authjwt_exception_handler(_request: Request, exc: AuthJWTException):
    return JSONResponse(
        status_code=exc.status_code,
        content={"detail": exc.message},
    )
