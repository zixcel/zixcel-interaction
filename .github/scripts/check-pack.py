"""Check the captured npm archive with typed, bounded validation."""
import json
import pathlib
import sys
from package_check.archive import check_archive


def main() -> None:
    """Only aggregate metadata reaches stdout; errors never contain matched values."""
    if len(sys.argv) != 2:
        raise ValueError("usage: check-pack.py ARCHIVE")
    print(json.dumps(check_archive(pathlib.Path(sys.argv[1]))))


if __name__ == "__main__":
    main()
