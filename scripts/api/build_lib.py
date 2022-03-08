#!/usr/bin/env python3

import argparse
import os
import shutil
from subprocess import Popen
from pathlib import Path
from typing import List


root_dir = Path(__file__).parent.parent.parent.absolute()
work_dir = root_dir / "api"
os.chdir(work_dir)


def run(*cmds: List[str]):
    p = Popen([str(arg) for arg in cmds])
    p.wait()
    if p.returncode != 0:
        exit(p.returncode)


parser = argparse.ArgumentParser()
parser.add_argument(
    "--debug",
    action="store_const",
    const="debug",
    default="release",
    dest="mode",
)
opts = parser.parse_args()

run("cargo", "build", "--package", "widget_pybind", f"--{opts.mode}")
shutil.copy2(
    root_dir / "target" / opts.mode / "libwidget_pybind.so",
    root_dir / "api" / "app" / "core",
)
os.system("cargo build --package widget_pybind --release")
