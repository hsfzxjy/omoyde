#!/usr/bin/env python3

import json
from pathlib import Path
from passlib.context import CryptContext

root_dir = Path(__file__).parent.parent.absolute()
config_file = root_dir / "config.json"


def setup_api_config_json_file():
    api_config_file = root_dir / "api" / "app" / "config.json"
    config_content = json.loads(config_file.read_text())

    pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")
    for field_name in {"pincode", "password"}:
        config_content["security"][field_name] = pwd_context.hash(
            config_content["security"][field_name]
        )

    api_config_file.write_text(json.dumps(config_content, indent=2))


if __name__ == "__main__":
    setup_api_config_json_file()
