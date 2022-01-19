from datetime import timedelta
from pydantic import BaseModel
from fastapi_jwt_auth import AuthJWT

from app.models.config import cfg


class JWTSettings(BaseModel):
    authjwt_secret_key: str
    authjwt_token_location: set = {"cookies"}
    authjwt_cookie_csrf_protect: bool = False
    authjwt_algorithms: set = {"HS256"}
    authjwt_decode_algorithms: set = {"HS256"}
    authjwt_access_token_expires: timedelta = timedelta(minutes=60)
    authjwt_refresh_token_expires: timedelta = timedelta(days=60)


@AuthJWT.load_config
def authjwt_load_config():
    return JWTSettings(authjwt_secret_key=cfg.security.secret)
