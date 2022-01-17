#!/usr/bin/env python3

import os
import json
import argparse
from pathlib import Path

root_dir = Path(__file__).parent.parent.parent.absolute()
work_dir = root_dir / "api"
os.chdir(work_dir)
cfg = json.loads((root_dir / "config.json").read_text())

parser = argparse.ArgumentParser()
parser.add_argument("-p", "--proxy", action="store_true", help="use proxy")
opts = parser.parse_args()

os.execvp(
    "docker",
    [
        "docker",
        "build",
        "--network=host",
        f'--tag={cfg["system"]["docker_name"]}',
        "--build-arg",
        f"USE_PROXY={1 if opts.proxy else ''}",
        ".",
    ],
)
