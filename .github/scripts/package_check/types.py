"""Archive receipts contain identity and hashes, never scanned source contents."""
from typing import TypedDict


class ArchiveReceipt(TypedDict):
    """Evidence for one immutable byte sequence, not a confidentiality proof."""
    name: str
    version: str
    sha256: str
    file_count: int
    bounded_scan: str
