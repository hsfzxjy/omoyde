#!/usr/bin/env python3
from dataclasses import dataclass
from typing import Literal, Optional
import argparse
import re
import json
from pathlib import Path
from subprocess import Popen, PIPE, DEVNULL
from wsgiref import headers

root_dir = Path(__file__).parent.parent.absolute()
config_file = root_dir / "config.json"
cfg = json.loads(config_file.read_text())
prefix = (
    f"{cfg['tcloud']['cos']['bucket']}.cos."
    f"{cfg['tcloud']['cos']['region']}.myqcloud.com"
)


def _escape_cli_args(args):
    ret = []
    for x in args:
        if x == None or x == "":
            continue
        if isinstance(x, dict):
            x = json.dumps(x)
        else:
            x = str(x)
        ret.append(x)
    return ret


@dataclass
class CmdResult:
    stdout: Optional[str]
    retcode: int


def runcmd(
    *args,
    raise_on_fail: bool = True,
    want_stdout: bool = False,
    stdin=None,
    accept=frozenset([0, 255]),
):
    args = _escape_cli_args(args)
    print(f'EXECUTING [{" ".join(map(repr, args))}]')
    p = Popen(
        args,
        stdin=PIPE if stdin else DEVNULL,
        stdout=PIPE if want_stdout else DEVNULL,
    )

    for line in stdin or []:
        p.communicate(bytes(line, encoding="utf-8"))
    p.wait()

    retcode = p.returncode
    if retcode not in accept and raise_on_fail:
        raise RuntimeError(f"Command {args} failed with return code {retcode}")

    return CmdResult(p.stdout.read().decode() if want_stdout else None, retcode)


def check_local():
    o = runcmd(root_dir / "scripts" / "butler", "index", want_stdout=True)
    if re.match(r"\s0 selected but missing, 0 selected but modified.", o.stdout):
        raise RuntimeError("local photos diverged")

    runcmd(root_dir / "scripts" / "butler", "generate")


def setup_coscmd():
    print("Setting up COS...")
    runcmd(
        "coscmd",
        "config",
        "-a",
        cfg["tcloud"]["secretId"],
        "-s",
        cfg["tcloud"]["secretKey"],
        "-b",
        cfg["tcloud"]["cos"]["bucket"],
        "-r",
        cfg["tcloud"]["cos"]["region"],
    )


def upload_generated_photos():
    print("Uploading assets/_generated ...")
    headers = {"Cache-Control": "private,max-age=15,must-revalidate"}
    for img_dir in opts.img_dirs:
        runcmd(
            "coscmd",
            "upload",
            "-H",
            headers,
            "--sync",
            "--delete",
            "--recursive",
            opts.skipmd5,
            f"assets/_generated/{img_dir}/",
            f"assets/{img_dir}/",
            stdin=["y"],
        )
        if opts.fix_headers:
            runcmd(
                "coscmd",
                "copy",
                "-H",
                headers,
                "--recursive",
                "-d",
                "Replaced",
                f"{prefix}/assets/{img_dir}",
                f"/assets/{img_dir}",
            )


@dataclass
class ListItem:
    path: str
    default_content: bytes
    public: bool
    respect: Literal["remote", "local"]

    @property
    def local_path(self) -> Path:
        return Path("assets/_generated") / self.path

    @property
    def remote_path(self) -> str:
        return f"assets/{self.path}"

    @property
    def migrated_local_path(self) -> Path:
        return Path("assets/_generated") / (self.path + ".migrated")

    def sync_myself(self):
        respect = self.respect
        if self.migrated_local_path.exists():
            ans = input(
                f"Are you sure to overwrite {self.local_path} "
                f"with {self.migrated_local_path}? (y/n) "
            )
            if ans == "y":
                self.migrated_local_path.rename(self.local_path)
                respect = "local"

        if runcmd("coscmd", "info", self.remote_path).retcode == 255:
            # object not exists on remote
            if not self.local_path.exists():
                self.local_path.write_bytes(self.default_content)
        elif respect == "remote":
            runcmd("coscmd", "download", "-f", self.remote_path, self.local_path)

        headers = {"Cache-Control": "private,max-age=0,must-revalidate"}
        runcmd(
            "coscmd",
            "upload",
            "--sync",
            "-H",
            headers,
            self.local_path,
            self.remote_path,
            accept={0, 255, 254},
        )
        if opts.fix_headers:
            runcmd(
                "coscmd",
                "copy",
                "-d",
                "Replaced",
                "-H",
                headers,
                f"{prefix}/{self.remote_path}",
                self.remote_path,
            )

        if self.public:
            runcmd("coscmd", "putobjectacl", "--grant-read", "anyone", self.remote_path)


ITEMS_TO_SYNC = [
    ListItem("metas.bin", b"", False, "local"),
    ListItem("msg.bin", b"", False, "remote"),
]


def sync_lists():
    for item in ITEMS_TO_SYNC:
        item.sync_myself()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--img-dirs", "-i", nargs="*", default=["m", "s", "source"])
    parser.add_argument("--skipmd5", action="store_const", const="--skipmd5")
    parser.add_argument("--fix-headers", action="store_true")
    opts = parser.parse_args()
    check_local()
    setup_coscmd()
    sync_lists()
    upload_generated_photos()
