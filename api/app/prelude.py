import aioredis
from pydantic import BaseModel
from passlib.context import CryptContext
from fastapi import Depends, HTTPException, status, FastAPI, Request
from fastapi_jwt_auth import AuthJWT
from fastapi_jwt_auth.exceptions import AuthJWTException

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
]

app = FastAPI(title="FastAPI Redis Tutorial")
redis = aioredis.from_url(cfg.redis_url, decode_responses=True)
pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")


@app.exception_handler(AuthJWTException)
def authjwt_exception_handler(_request: Request, exc: AuthJWTException):
    return JSONResponse(
        status_code=exc.status_code,
        content={"detail": exc.message},
    )
