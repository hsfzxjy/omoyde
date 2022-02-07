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


def setup_web_vue_env_file():
    config_content = json.loads(config_file.read_text())

    # don't transcribe entries under "local"
    del config_content["local"]

    entries = {}

    def _transcribe(cfg: dict, prefix=("VITE",)):
        for k, v in cfg.items():
            k = k.upper()
            if isinstance(v, dict):
                _transcribe(v, prefix + (k,))
            else:
                key = "_".join(prefix + (k,))
                entries[key] = v

    def _to_file(file: Path):
        with file.open("w") as fd:
            for k, v in sorted(entries.items()):
                fd.write(f"{k}={v}\n")

    _transcribe(config_content)
    _to_file(root_dir / "web" / ".env")
    config_content["web"]["authURL"] = "http://localhost:8080"
    _transcribe(config_content)
    _to_file(root_dir / "web" / ".env.development.local")


if __name__ == "__main__":
    setup_api_config_json_file()
    setup_web_vue_env_file()
