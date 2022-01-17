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
parser.add_argument("action", choices=["dev", "shell"])
parser.add_argument("-p", "--port", type=int, default=8080, help="port to listen on")
opts = parser.parse_args()

docker_cmd = {
    "dev": "/start-reload.sh",
    "shell": "bash",
}[opts.action]

os.execvp(
    "docker",
    [
        "docker",
        "run",
        "-it",
        "--rm",
        "-p",
        f"{opts.port}:80",
        "-v",
        f'{work_dir / "app"}:/app',
        cfg["system"]["docker_name"],
        docker_cmd,
    ],
)
