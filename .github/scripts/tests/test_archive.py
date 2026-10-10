"""Synthetic archives exercise rejection without extracting files or using accounts."""
import io
import json
import pathlib
import subprocess
import sys
import tarfile
import tempfile
import unittest
from package_check.archive import check_archive


class ArchiveTests(unittest.TestCase):
    """Verify explicit failures and the exact immutable-byte receipt."""
    def archive(self, root: pathlib.Path, extra: tuple[str, bytes] | None = None,
                private: bool = False, symlink: bool = False) -> pathlib.Path:
        path = root / "fixture.tgz"
        manifest = json.dumps({"name": "fixture", "version": "0.0.0", "private": private,
                               "exports": "./index.mjs"}).encode()
        entries = [("package/package.json", manifest), ("package/LICENSE", b"fixture"),
                   ("package/index.mjs", b"export const value = false;")]
        if extra is not None:
            entries.append(extra)
        with tarfile.open(path, "w:gz") as archive:
            for name, content in entries:
                entry = tarfile.TarInfo(name); entry.size = len(content)
                if symlink and name == "package/index.mjs":
                    entry.type = tarfile.SYMTYPE; entry.linkname = "outside"
                archive.addfile(entry, io.BytesIO(content))
        return path

    def test_reviewed_archive_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            result = check_archive(self.archive(pathlib.Path(directory)))
            self.assertEqual(result["name"], "fixture")
            self.assertEqual(result["file_count"], 3)
            self.assertEqual(len(result["sha256"]), 64)

    def test_unsafe_paths_and_duplicates(self) -> None:
        for name in ("package/../outside", "/package/absolute", "package/index.mjs"):
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                with self.assertRaises(ValueError):
                    check_archive(self.archive(pathlib.Path(directory), (name, b"data")))

    def test_links_and_private_packages(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            for private, symlink in ((True, False), (False, True)):
                with self.subTest(private=private, symlink=symlink), self.assertRaises(ValueError):
                    check_archive(self.archive(root, private=private, symlink=symlink))

    def test_configuration_and_malformed_key_text(self) -> None:
        for name, value in (("package/.env", b"fixture"),
                            ("package/key.txt", b"-----BEGIN PRIVATE KEY-----\nnot-a-key")):
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                with self.assertRaises(ValueError):
                    check_archive(self.archive(pathlib.Path(directory), (name, value)))

    def test_size_checked_before_reading_member_body(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "oversize.tgz"
            with tarfile.open(path, "w:gz") as archive:
                entry = tarfile.TarInfo("package/oversize"); entry.size = 16_777_217
                archive.addfile(entry)
            with self.assertRaisesRegex(ValueError, "expanded archive size"):
                check_archive(path)

    def test_optimized_python_rejects_private_archive(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = self.archive(pathlib.Path(directory), private=True)
            script = pathlib.Path(__file__).resolve().parents[1] / "check-pack.py"
            result = subprocess.run([sys.executable, "-O", str(script), str(path)],
                                    capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertNotIn('"bounded_scan"', result.stdout)


if __name__ == "__main__":
    unittest.main()
