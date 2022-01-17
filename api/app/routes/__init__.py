from . import auth

from ..prelude import *


@app.get("/secret")
def secret(Authorize: AuthJWT = Depends()):
    Authorize.jwt_required()
    return "whoops"
