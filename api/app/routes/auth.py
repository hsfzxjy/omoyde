from pydantic import BaseModel

from app.prelude import *


class LoginPayload(BaseModel):
    password: str


SUBJECT_ADMIN = "admin"


@app.post("/login")
def login(payload: LoginPayload, Authorize: AuthJWT = Depends()):
    password = payload.password
    if not pwd_context.verify(password, cfg.security.password):
        raise HTTPException(status.HTTP_401_UNAUTHORIZED, "password mismatched")

    access_token = Authorize.create_access_token(subject=SUBJECT_ADMIN)
    refresh_token = Authorize.create_refresh_token(subject=SUBJECT_ADMIN)

    Authorize.set_access_cookies(access_token)
    Authorize.set_refresh_cookies(refresh_token)

    return "ok"


@app.post("/refresh")
def refresh(payload: LoginPayload, Authorize: AuthJWT = Depends()):
    Authorize.jwt_refresh_token_required()

    if not pwd_context.verify(payload.password, cfg.security.pincode):
        raise HTTPException(status.HTTP_401_UNAUTHORIZED, "pincode mismatched")

    new_access_token = Authorize.create_access_token(subject=SUBJECT_ADMIN)
    Authorize.set_access_cookies(new_access_token)
    return "ok"
