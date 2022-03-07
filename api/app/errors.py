from fastapi import HTTPException
from fastapi_jwt_auth.exceptions import AuthJWTException
from qcloud_cos import CosClientError

__all__ = [
    "OmoydeException",
    "ClientFileTooOld",
    "CosClientError",
    "get_error_response",
]


class OmoydeException(Exception):
    status_code = 400


class ClientFileTooOld(OmoydeException):
    detail = "client file is too old"


_EXC_TO_CODE = {
    ClientFileTooOld: "E1001",
}


def get_error_response(exc: Exception) -> str:
    if isinstance(exc, HTTPException):
        return {
            "code": f"E9{exc.status_code}",
            "detail": exc.detail,
        }

    if isinstance(exc, AuthJWTException):
        return {
            "code": "E1000",
            "detail": exc.message,
        }

    if isinstance(exc, CosClientError):
        return {
            "code": "E2000",
            "detail": exc._message,
        }

    return {
        "code": _EXC_TO_CODE[exc.__class__],
        "detail": exc.detail,
    }
