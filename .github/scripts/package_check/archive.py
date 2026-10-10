"""Bound extraction before decompression and inspect a single captured byte sequence."""
import hashlib
import io
import json
import pathlib
import re
import tarfile
from typing import cast
from .manifest import check_exports, fields, require
from .types import ArchiveReceipt

MAXIMUM_ARCHIVE_BYTES = 67_108_864
MAXIMUM_FILE_BYTES = 16_777_216
PATTERNS = (
    r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----",
    r"gh[pousr]_[A-Za-z0-9]{20,}", r"github_pat_[A-Za-z0-9_]{20,}",
    r"npm_[A-Za-z0-9]{20,}", r"AKIA[A-Z0-9]{16}", r"/(?:home|mnt)/",
    r"[A-Z]:[\\/](?:Users|Documents)[\\/]", r"https?://[^/\s:@]+:[^/\s@]+@",
)


def check_archive(path: pathlib.Path) -> ArchiveReceipt:
    """Never extract files or include matched data in errors or receipts."""
    with path.open("rb") as source:
        data = source.read(MAXIMUM_ARCHIVE_BYTES + 1)
    require(len(data) <= MAXIMUM_ARCHIVE_BYTES, "archive size limit")
    names: set[str] = set()
    total = 0
    manifest_bytes: bytes | None = None
    # Streaming traversal checks each size before seeking/decompressing its body.
    with tarfile.open(fileobj=io.BytesIO(data), mode="r|gz") as archive:
        for member in archive:
            path_parts = pathlib.PurePosixPath(member.name)
            require(not path_parts.is_absolute() and ".." not in path_parts.parts
                    and bool(path_parts.parts) and path_parts.parts[0] == "package",
                    "unsafe archive path")
            require(member.isfile() or member.isdir(), "unsupported archive member")
            require(str(path_parts) not in names and "\\" not in member.name,
                    "duplicate or ambiguous archive path")
            names.add(str(path_parts))
            total += member.size
            require(len(names) <= 10_000 and 0 <= member.size <= MAXIMUM_FILE_BYTES
                    and total <= MAXIMUM_ARCHIVE_BYTES, "expanded archive size limit")
            require(not any(part == ".npmrc" or part.startswith(".env")
                            for part in path_parts.parts), "private archive configuration")
            if not member.isfile():
                continue
            stream = archive.extractfile(member)
            if stream is None:
                raise ValueError("archive member unreadable")
            content = stream.read(MAXIMUM_FILE_BYTES + 1)
            require(len(content) == member.size, "archive member size mismatch")
            text = content.decode(errors="replace")
            require(not any(re.search(pattern, text, re.I) for pattern in PATTERNS),
                    "bounded archive disclosure scan finding")
            if str(path_parts) == "package/package.json":
                manifest_bytes = content
    require(bool(names) and "package/LICENSE" in names, "archive license missing")
    if manifest_bytes is None:
        raise ValueError("archive manifest missing")
    manifest = fields(cast(object, json.loads(manifest_bytes)))
    require(manifest.get("private", False) is False, "private archive package")
    name, version = manifest.get("name"), manifest.get("version")
    if not isinstance(name, str) or not isinstance(version, str):
        raise ValueError("archive identity must contain strings")
    check_exports(manifest.get("exports"), names)
    return {"name": name, "version": version,
            "sha256": hashlib.sha256(data).hexdigest(), "file_count": len(names),
            "bounded_scan": "passed; not a confidentiality proof"}
