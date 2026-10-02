#!/usr/bin/env python3
"""Bounded, read-only primitives shared by Forge dependency tooling."""
from __future__ import annotations

import hashlib
import json
import math
import os
from pathlib import Path, PurePosixPath
import re
import stat
import tomllib
from typing import Any

MAX_INPUT_BYTES = 16 * 1024 * 1024
MAX_JSON_DEPTH = 64


class InputError(ValueError):
    """A committed input cannot safely be interpreted."""


def canonical_bytes(value: Any) -> bytes:
    try:
        return (json.dumps(value, sort_keys=True, separators=(",", ":"),
                           ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")
    except (ValueError, TypeError, RecursionError) as exc:
        raise InputError("value is not canonical JSON") from exc


def identity(pkg: dict[str, Any]) -> tuple[str, str, str]:
    return (pkg["name"], pkg["version"], pkg.get("source") or "workspace")


def package_id(pkg: dict[str, Any]) -> str:
    return json.dumps(list(identity(pkg)), separators=(",", ":"), ensure_ascii=True)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _relative(relative: str) -> tuple[str, ...]:
    if not isinstance(relative, str) or not relative or "\\" in relative:
        raise InputError("input path must be a repository-relative POSIX path")
    if any(ord(char) < 32 or ord(char) == 127 for char in relative):
        raise InputError("input path contains controls")
    path = PurePosixPath(relative)
    parts = relative.split("/")
    if path.is_absolute() or any(part in ("", ".", "..") for part in parts):
        raise InputError("input path must not traverse or alias directories")
    reserved = {"CON", "PRN", "AUX", "NUL", *(f"COM{i}" for i in range(1, 10)),
                *(f"LPT{i}" for i in range(1, 10))}
    if any(":" in part or part.endswith((".", " "))
           or part.split(".", 1)[0].upper() in reserved for part in parts):
        raise InputError("input path contains a Windows drive, stream or reserved alias")
    return tuple(parts)


def _plain(info: os.stat_result) -> None:
    if getattr(info, "st_file_attributes", 0) & 0x400 or getattr(info, "st_reparse_tag", 0):
        raise InputError("input path is a Windows reparse point")


def _regular(info: os.stat_result, limit: int) -> None:
    _plain(info)
    if not stat.S_ISREG(info.st_mode):
        raise InputError("input is not a regular file")
    if info.st_nlink != 1:
        raise InputError("input has a hard-link alias")
    if info.st_size > limit:
        raise InputError(f"input exceeds {limit} byte limit")


def _same(before: os.stat_result, after: os.stat_result) -> bool:
    return (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns,
            before.st_ctime_ns) == (after.st_dev, after.st_ino, after.st_size,
                                    after.st_mtime_ns, after.st_ctime_ns)


def read_bytes(root: Path | str, relative: str,
               limit: int = MAX_INPUT_BYTES) -> bytes:
    """Reject links/special files before bounded reads; anchor parents on POSIX.

    Windows uses no-follow stat and file-identity comparisons; this is validation
    of a committed checkout, not a general filesystem sandbox against writers.
    """
    parts = _relative(relative)
    if type(limit) is not int or limit < 1 or limit > MAX_INPUT_BYTES:
        raise InputError("invalid input byte limit")
    fds: list[int] = []
    try:
        base = Path(root).resolve(strict=True)
        if not base.is_dir():
            raise InputError("repository root is not a directory")
        anchored = (os.open in os.supports_dir_fd and os.stat in os.supports_dir_fd
                    and hasattr(os, "O_NOFOLLOW") and hasattr(os, "O_DIRECTORY"))
        if anchored:
            dir_flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
            parent = os.open(base, dir_flags)
            fds.append(parent)
            for part in parts[:-1]:
                parent = os.open(part, dir_flags, dir_fd=parent)
                fds.append(parent)
            before = os.stat(parts[-1], dir_fd=parent, follow_symlinks=False)
            _regular(before, limit)
            flags = os.O_RDONLY | os.O_NOFOLLOW | getattr(os, "O_NONBLOCK", 0)
            fd = os.open(parts[-1], flags, dir_fd=parent)
        else:
            path = base
            for part in parts[:-1]:
                path = path / part
                info = path.lstat()
                _plain(info)
                if not stat.S_ISDIR(info.st_mode):
                    raise InputError("input parent is not a plain directory")
            path = path / parts[-1]
            before = path.lstat()
            _regular(before, limit)
            fd = os.open(path, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0)
                         | getattr(os, "O_NOFOLLOW", 0))
        fds.append(fd)
        opened = os.fstat(fd)
        _regular(opened, limit)
        if not _same(before, opened):
            raise InputError("input changed while opening")
        chunks: list[bytes] = []
        size = 0
        while size <= limit:
            chunk = os.read(fd, min(65536, limit + 1 - size))
            if not chunk:
                break
            chunks.append(chunk)
            size += len(chunk)
        if size > limit:
            raise InputError(f"input exceeds {limit} byte limit")
        after = os.fstat(fd)
        if anchored:
            current = os.stat(parts[-1], dir_fd=parent, follow_symlinks=False)
        else:
            current = path.lstat()
        if not _same(opened, after) or not _same(after, current) or size != after.st_size:
            raise InputError("input changed while reading")
        return b"".join(chunks)
    except (OSError, RuntimeError) as exc:
        raise InputError(f"cannot read {relative} (filesystem error)") from exc
    finally:
        for fd in reversed(fds):
            os.close(fd)


def _json_depth(text: str) -> None:
    depth = 0
    quoted = False
    escaped = False
    for char in text:
        if quoted:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                quoted = False
        elif char == '"':
            quoted = True
        elif char in "[{":
            depth += 1
            if depth > MAX_JSON_DEPTH:
                raise InputError(f"JSON nesting exceeds {MAX_JSON_DEPTH}")
        elif char in "]}":
            depth -= 1


def load_json(data: bytes) -> Any:
    if len(data) > MAX_INPUT_BYTES:
        raise InputError("JSON exceeds input byte limit")
    try:
        text = data.decode("utf-8")
        _json_depth(text)
        def pairs(values: list[tuple[str, Any]]) -> dict[str, Any]:
            result = {}
            for key, value in values:
                if key in result:
                    raise InputError("JSON contains a duplicate object key")
                result[key] = value
            return result
        def bad_constant(value: str) -> Any:
            raise InputError("JSON contains a non-finite number")
        def finite_float(value: str) -> float:
            result = float(value)
            if not math.isfinite(result):
                raise InputError("JSON contains an overflowing non-finite number")
            return result
        return json.loads(text, object_pairs_hook=pairs, parse_constant=bad_constant,
                          parse_float=finite_float)
    except InputError:
        raise
    except (UnicodeDecodeError, json.JSONDecodeError, ValueError, RecursionError) as exc:
        raise InputError("invalid UTF-8 JSON") from exc


def load_toml(data: bytes) -> dict[str, Any]:
    if len(data) > MAX_INPUT_BYTES:
        raise InputError("TOML exceeds input byte limit")
    try:
        result = tomllib.loads(data.decode("utf-8"))
        # Bound authored container nesting independently of the TOML decoder.
        pending = [(result, 0)]
        while pending:
            value, depth = pending.pop()
            if depth > MAX_JSON_DEPTH:
                raise InputError(f"TOML nesting exceeds {MAX_JSON_DEPTH}")
            if isinstance(value, dict):
                pending.extend((child, depth + 1) for child in value.values())
            elif isinstance(value, list):
                pending.extend((child, depth + 1) for child in value)
        return result
    except InputError:
        raise
    except (UnicodeDecodeError, tomllib.TOMLDecodeError, ValueError, RecursionError) as exc:
        raise InputError("invalid UTF-8 TOML") from exc


def read_release_targets(data: bytes) -> list[str]:
    """Read the deliberately narrow, literal release build matrix contract.

    A workflow schema change requires review of this parser and graph refresh.
    No YAML dependency or expression evaluation is used.
    """
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise InputError("release workflow is not UTF-8") from exc
    if "\t" in text:
        raise InputError("release workflow contains tabs")
    lines = text.splitlines()
    starts = [index for index, line in enumerate(lines) if line == "  build:"]
    if len(starts) != 1 or "jobs:" not in lines[:starts[0]]:
        raise InputError("release workflow needs one literal build job")
    start = starts[0] + 1
    stop = next((i for i in range(start, len(lines))
                 if re.fullmatch(r"  [A-Za-z0-9_-]+:.*", lines[i])), len(lines))
    block = lines[start:stop]
    strategies = [i for i, line in enumerate(block) if line == "    strategy:"]
    if len(strategies) != 1:
        raise InputError("release build requires one literal strategy")
    strategy_start = strategies[0] + 1
    strategy_stop = next((i for i in range(strategy_start, len(block))
                         if block[i].strip() and not block[i].lstrip().startswith("#")
                         and not block[i].startswith("      ")), len(block))
    strategy = block[strategy_start:strategy_stop]
    matrices = [i for i, line in enumerate(strategy) if line == "      matrix:"]
    if len(matrices) != 1:
        raise InputError("release build requires one literal strategy matrix")
    matrix_start = matrices[0] + 1
    matrix_stop = next((i for i in range(matrix_start, len(strategy))
                        if strategy[i].strip() and not strategy[i].lstrip().startswith("#")
                        and not strategy[i].startswith("        ")), len(strategy))
    matrix = [line for line in strategy[matrix_start:matrix_stop]
              if line.strip() and not line.lstrip().startswith("#")]
    if not matrix or matrix[0] != "        include:":
        raise InputError("release build requires explicit matrix.include")
    rows = []
    row = None
    for line in matrix[1:]:
        target = re.fullmatch(r"          - target: ([a-z0-9_]+(?:-[a-z0-9_]+)+)", line)
        field = re.fullmatch(r"            (os|archive): ([A-Za-z0-9_.-]+)", line)
        if target:
            row = {"target": target.group(1)}
            rows.append(row)
        elif field and row is not None and field.group(1) not in row:
            row[field.group(1)] = field.group(2)
        else:
            raise InputError("unsupported or duplicate release build matrix entry")
    if any(set(row) != {"target", "os", "archive"} for row in rows):
        raise InputError("release target needs unique os and archive fields")
    targets = [row["target"] for row in rows]
    if not targets or len(targets) != len(set(targets)) or len(targets) > 32:
        raise InputError("release targets empty, duplicate or excessive")
    if any(re.search(r"--(?:all-features|no-default-features|features)(?:[ =]|$)", line)
           for line in block if not line.lstrip().startswith("#")):
        raise InputError("release feature flags require a new graph feature contract")

    return sorted(targets)


def release_targets(root: Path | str) -> list[str]:
    return read_release_targets(read_bytes(root, ".github/workflows/release.yml"))
