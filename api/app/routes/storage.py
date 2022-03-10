import json
from sts.sts import Sts
from qcloud_cos import CosServiceError

from app.prelude import *
from app.core import _widget

REDIS_KEY_COS_CREDENTIAL = ":sts:crendential"
OBJECT_HEADERS = {
    "CacheControl": "private,max-age=0,must-revalidate",
}


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


async def mod_widgets(mods, expected_hash):
    try:
        r = cos_client.get_object(
            cfg.tcloud.cos.bucket, "/assets/widgets.bin", IfMatch=expected_hash
        )
    except CosServiceError as e:
        if e.get_error_code() == "PreconditionFailed":
            raise ClientFileTooOld()
        raise e
    old_items = bytearray(b"").join(r["Body"])
    new_items = _widget.mod_widgets(
        _widget.FFIVec.from_bytes(old_items),
        _widget.FFIVec.from_bytes(mods),
    ).contents
    with new_items.guard():
        # _widget.display_widgets(_widget.FFIVec.from_bytes(old_items))
        # _widget.display_widgets(new_items)
        new_items = new_items.to_bytes()

    r = cos_client.put_object(
        cfg.tcloud.cos.bucket, new_items, "/assets/widgets.bin", **OBJECT_HEADERS
    )


@app.post("/storage/mod_widgets")
async def storage_mod_widgets(
    Authorize: AuthJWT = Depends(),
    mods=Body(...),
    expected_hash=Header(None),
):
    Authorize.jwt_required()
    await mod_widgets(mods, expected_hash)
    return "ok"
