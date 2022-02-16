from typing import Literal
import json
from sts.sts import Sts

from app.prelude import *

REDIS_KEY_COS_CREDENTIAL = ":sts:crendential"


async def get_cos_credential():
    cred = await redis.get(REDIS_KEY_COS_CREDENTIAL)
    if cred is not None:
        return json.loads(cred)
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
    cred = dict(sts.get_credential())
    await redis.set(REDIS_KEY_COS_CREDENTIAL, json.dumps(cred), ex=3540)
    return cred


@app.get("/storage/credential")
async def storage_get_credential(Authorize: AuthJWT = Depends()):
    Authorize.jwt_required()
    return await get_cos_credential()
