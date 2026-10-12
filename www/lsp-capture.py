"""www/lsp-capture.py — what the Typst language server shows on a task file.

www/generate.sh runs `python3 www/lsp-capture.py <repo> <out>`. The script
starts `tinymist lsp` from the dev shell, speaks the Language Server Protocol
to it over stdio, and writes one record, <out>/lsp.json, for the Overview: the
fields tinymist offers inside `task.with(...)`, and what it reports on the
same file, which writes `titel:`, and where.

The fixture is examples/lsp, one task file and no project, so `mt` never
reads the misspelled file. The file marks the cursor with a `// docs:cursor`
line, which this script removes before it opens the file.

tinymist finds `@local/mindtape` where a reader's tinymist finds it: in the
data home `mt install` writes to. The install goes to a temporary data home,
from the `mt` this build compiled, so the prelude is this checkout's lib/.
Fonts and caches are pinned too, so the records depend on the prelude and the
tinymist version alone.

The build fails when tinymist offers no field, does not answer within the
timeout, or reports a problem anywhere but the task file, or none there.
"""

import json
import os
import queue
import shutil
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path
from urllib.parse import unquote, urlparse

FIXTURE = "examples/lsp"
TASK = "tasks/fix-the-login-redirect.typ"
CURSOR = "// docs:cursor"
TIMEOUT = 60
SEVERITY = {1: "error", 2: "warning", 3: "info", 4: "hint"}
# LSP CompletionItemKind.Field: tinymist's kind for a named argument.
FIELD_KIND = 5


class CaptureError(Exception):
    pass


def fail(message):
    raise CaptureError(message)


class Server:
    """One `tinymist lsp` process and the JSON-RPC framing around it."""

    def __init__(self, env, log):
        self.proc = subprocess.Popen(
            ["tinymist", "lsp", "--ignore-system-fonts"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=log,
            env=env,
        )
        self.inbox = queue.Queue()
        self.next_id = 0
        # Diagnostics tinymist last published, by URI.
        self.diagnostics = {}
        threading.Thread(target=self._read, daemon=True).start()

    def _read(self):
        stream = self.proc.stdout
        while True:
            length = None
            while True:
                line = stream.readline()
                if not line:
                    self.inbox.put(None)
                    return
                line = line.decode("ascii").strip()
                if not line:
                    break
                name, _, value = line.partition(":")
                if name.lower() == "content-length":
                    length = int(value)
            self.inbox.put(json.loads(stream.read(length)))

    def _send(self, message):
        body = json.dumps({"jsonrpc": "2.0", **message}).encode()
        self.proc.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
        self.proc.stdin.flush()

    def notify(self, method, params):
        self._send({"method": method, "params": params})

    def _take(self, deadline):
        try:
            message = self.inbox.get(timeout=max(0, deadline - time.monotonic()))
        except queue.Empty:
            fail(f"tinymist sent nothing for {TIMEOUT}s")
        if message is None:
            fail("tinymist exited")
        if message.get("method") == "textDocument/publishDiagnostics":
            params = message["params"]
            self.diagnostics[params["uri"]] = params["diagnostics"]
        elif "method" in message and "id" in message:
            # A request from the server, such as workspace/configuration.
            # Every setting keeps its default.
            items = message.get("params", {}).get("items")
            result = [None] * len(items) if items else None
            self._send({"id": message["id"], "result": result})
        return message

    def request(self, method, params):
        self.next_id += 1
        self._send({"id": self.next_id, "method": method, "params": params})
        deadline = time.monotonic() + TIMEOUT
        while True:
            message = self._take(deadline)
            if message.get("id") == self.next_id and "method" not in message:
                if "error" in message:
                    fail(f"{method}: {message['error']}")
                return message.get("result")

    def wait_for_diagnostics(self):
        deadline = time.monotonic() + TIMEOUT
        while not any(self.diagnostics.values()):
            self._take(deadline)

    def open(self, uri, text):
        self.notify(
            "textDocument/didOpen",
            {
                "textDocument": {
                    "uri": uri,
                    "languageId": "typst",
                    "version": 1,
                    "text": text,
                }
            },
        )


def install_prelude(repo, data_home):
    """Install this build's prelude the way a reader does, into data_home."""
    env = dict(os.environ, MINDTAPE_DATA_HOME=str(data_home), MINDTAPE_CONFIG_HOME="")
    subprocess.run(
        [str(repo / "target/debug/mt"), "install", "--no-defaults", "--no-schema", "--no-completions"],
        env=env,
        check=True,
        stdout=subprocess.DEVNULL,
    )
    return data_home / "typst" / "packages"


def server_env(tmp):
    env = dict(os.environ, XDG_DATA_HOME=str(tmp / "data"), XDG_CACHE_HOME=str(tmp / "cache"))
    for name in ("TYPST_PACKAGE_PATH", "TYPST_PACKAGE_CACHE_PATH", "TYPST_FONT_PATHS", "TYPST_ROOT"):
        env.pop(name, None)
    return env


def cursor_source(text):
    """The text with the cursor line emptied to its indent, and the cursor."""
    lines = text.split("\n")
    hits = [n for n, line in enumerate(lines) if line.strip() == CURSOR]
    if len(hits) != 1:
        fail(f"{TASK} needs one `{CURSOR}` line, found {len(hits)}")
    line = hits[0]
    column = lines[line].index(CURSOR)
    lines[line] = lines[line][:column]
    return "\n".join(lines), line, column


def fields(result):
    """The named arguments among the completion items, sorted by name."""
    items = result["items"] if isinstance(result, dict) else result or []
    names = []
    for item in items:
        edit = item.get("textEdit", {}).get("newText") or item.get("insertText") or ""
        if item.get("kind") == FIELD_KIND and edit.startswith(item["label"] + ":"):
            names.append(item["label"])
    return sorted(set(names))


def diagnostics(published, project):
    """The problems tinymist published for the task file, where it put them.
    A problem published for any other file is one an editor would not draw in
    the file the reader wrote, so the build fails on it."""
    out = []
    for uri, entries in published.items():
        path = Path(unquote(urlparse(uri).path))
        if entries and path != project / TASK:
            fail(f"tinymist reported {entries[0]['message']!r} on {uri}, not on {TASK}")
        for entry in entries:
            start, end = entry["range"]["start"], entry["range"]["end"]
            out.append(
                {
                    "file": TASK,
                    "line": start["line"] + 1,
                    "column": start["character"] + 1,
                    "end_line": end["line"] + 1,
                    "end_column": end["character"] + 1,
                    "severity": SEVERITY.get(entry.get("severity"), "error"),
                    "message": entry["message"],
                }
            )
    if not out:
        fail(f"tinymist reported nothing on {TASK}")
    return sorted(out, key=lambda d: (d["line"], d["column"], d["message"]))


def capture(repo, tmp):
    project = (repo / FIXTURE).resolve()
    packages = install_prelude(repo, tmp / "data")
    if not packages.is_dir():
        fail(f"mt install wrote no package directory at {packages}")
    with open(tmp / "tinymist.log", "w") as log:
        server = Server(server_env(tmp), log)
        info = server.request(
            "initialize",
            {
                "processId": os.getpid(),
                "rootUri": project.as_uri(),
                "capabilities": {"textDocument": {"publishDiagnostics": {}}},
            },
        ).get("serverInfo", {})
        server.notify("initialized", {})
        name = f"{info.get('name', 'tinymist')} {info.get('version', '')}".strip()

        uri = (project / TASK).as_uri()
        source, line, column = cursor_source((project / TASK).read_text())
        server.open(uri, source)
        offered = fields(
            server.request(
                "textDocument/completion",
                {
                    "textDocument": {"uri": uri},
                    "position": {"line": line, "character": column},
                },
            )
        )
        if not offered:
            fail(f"tinymist offered no field at the cursor in {TASK}")
        server.wait_for_diagnostics()
        server.request("shutdown", None)
        server.notify("exit", None)
        server.proc.wait(timeout=TIMEOUT)
    return {
        "server": name,
        "file": TASK,
        "source": source,
        "line": line + 1,
        "column": column + 1,
        "items": offered,
        "diagnostics": diagnostics(server.diagnostics, project),
    }


def main():
    repo, out = (Path(arg).resolve() for arg in sys.argv[1:3])
    tmp = Path(tempfile.mkdtemp())
    try:
        record = capture(repo, tmp)
    except CaptureError as error:
        # The end of tinymist's log first, then the reason, so the reason is
        # the last line the builder reads.
        log = tmp / "tinymist.log"
        if log.exists():
            sys.stderr.writelines(log.read_text().splitlines(True)[-20:])
        sys.exit(f"lsp-capture: {error}")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    (out / "lsp.json").write_text(json.dumps(record, indent=2) + "\n")


main()
