from typing import Literal
from sts.sts import Sts

from app.prelude import *


def get_cos_credential():
    config = {
        "duration_seconds": 3600,
        "secret_id": cfg.tcloud.secretId,
        "secret_key": cfg.tcloud.secretKey,
        "bucket": cfg.tcloud.cos.bucket,
        "region": cfg.tcloud.cos.region,
        "allow_prefix": "assets/*",
        "allow_actions": [
            "name/cos:GetObject",
            "name/cos:PutObject",
            "name/cos:HeadObject",
        ],
    }

    # propagate the exception, if any
    sts = Sts(config)
    return dict(sts.get_credential())


@app.get("/storage/credential")
def storage_get_credential(Authorize: AuthJWT = Depends()):
    Authorize.jwt_required()
    return get_cos_credential()
