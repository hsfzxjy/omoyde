#!/usr/bin/env python3

import typing as t
from pathlib import Path
import fastapi
from fastapi.responses import StreamingResponse
from fastapi.staticfiles import StaticFiles
import asyncio
from asyncio.subprocess import PIPE

# fmt: off
# We monkey-patch `StaticFiles.lookup_path` to enable
# serving symbolic links
import os, anyio
async def lookup_path(self, path):
    for directory in self.all_directories:
        full_path = os.path.realpath(os.path.join(directory, path))
        directory = os.path.realpath(directory)
        # NOTE: the commonprefix checking is removed
        try:
            stat_result = await anyio.to_thread.run_sync(os.stat, full_path)
            return full_path, stat_result
        except FileNotFoundError:
            pass
    return "", None
StaticFiles.lookup_path = lookup_path
# fmt: on

# important paths
cur_dir = Path(__file__).parent
root_dir = cur_dir.parent.absolute()

# FastAPI instance
app = fastapi.FastAPI()
app.mount("/web", StaticFiles(directory=cur_dir / "static" / "picker"), name="static")
app.mount("/img", StaticFiles(directory=root_dir / ".butler" / "links"), name="images")


@app.post("/cmd")
async def cmd(args: t.List[str]):
    p = await asyncio.subprocess.create_subprocess_exec(
        cur_dir / "butler", *args, stdout=PIPE
    )

    async def reader():
        while True:
            content = await p.stdout.read()
            if not content:
                break
            yield content
        await p.wait()

    return StreamingResponse(reader(), media_type="text/plain")


if __name__ == "__main__":
    import os
    import sys
    import webbrowser

    os.chdir(cur_dir)
    webbrowser.open("http://localhost:31415/web/index.html")

    os.execv(
        sys.executable,
        [
            "python3",
            "-m",
            "uvicorn",
            "--host=127.0.0.1",
            "--port=31415",
            "--reload",
            "picker:app",
        ],
    )
