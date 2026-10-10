"""Validate decoded manifest fields and exact package export paths."""
import pathlib
from typing import cast


def require(condition: bool, message: str) -> None:
    """Keep rejection active with python -O and PYTHONOPTIMIZE."""
    if not condition:
        raise ValueError(message)


def fields(value: object) -> dict[str, object]:
    """Narrow an external JSON object after validating its key representation."""
    require(isinstance(value, dict), "manifest must be an object")
    require(all(isinstance(key, str) for key in cast(dict[object, object], value)),
            "manifest keys must be strings")
    return cast(dict[str, object], value)


def check_exports(value: object, names: set[str]) -> None:
    """Every export must resolve to a reviewed, present file without wildcards."""
    if isinstance(value, str):
        require(value.startswith("./") and ".." not in pathlib.PurePosixPath(value).parts
                and "*" not in value, "invalid export path")
        require("package/" + value[2:] in names, "export file missing")
    elif isinstance(value, dict):
        for child in fields(value).values():
            check_exports(child, names)
    else:
        require(value is None, "invalid export definition")
