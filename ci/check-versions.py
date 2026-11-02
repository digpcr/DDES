"""Fail when package metadata or the supported Rust toolchain drifts."""

import json
import os
from pathlib import Path
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]


def toml(path: str) -> dict:
    return tomllib.loads((ROOT / path).read_text())


class MetadataTest(unittest.TestCase):
    def test_package_versions(self) -> None:
        if (ROOT / "python").is_dir():
            self.assertEqual(toml("python/pyproject.toml")["project"]["version"],
                             toml("python/Cargo.toml")["package"]["version"])
        if (ROOT / "typescript").is_dir():
            self.assertEqual(json.loads((ROOT / "typescript/package.json").read_text())["version"],
                             toml("typescript/Cargo.toml")["package"]["version"])
        if (ROOT / "r").is_dir():
            description = dict(line.split(": ", 1) for line in (ROOT / "r/DESCRIPTION").read_text().splitlines()
                               if line and not line.startswith(" ") and ": " in line)
            self.assertEqual(description["Version"], toml("r/src/rust/Cargo.toml")["package"]["version"])

    def test_bindings_use_this_core_and_toolchain(self) -> None:
        core = toml("ddes-rs/Cargo.toml")["package"]
        channel = toml("ddes-rs/rust-toolchain.toml")["toolchain"]["channel"]
        self.assertIn(os.environ.get("RUSTUP_TOOLCHAIN", channel), (channel, "stable"))
        self.assertTrue(channel.startswith(core["rust-version"] + "."))
        for manifest, toolchain in [("python/Cargo.toml", "python/rust-toolchain.toml"),
                                    ("typescript/Cargo.toml", "typescript/rust-toolchain.toml"),
                                    ("r/src/rust/Cargo.toml", "r/rust-toolchain.toml")]:
            if not (ROOT / Path(manifest).parts[0]).is_dir():
                continue
            with self.subTest(manifest=manifest):
                package = toml(manifest)
                self.assertEqual(package["package"]["rust-version"], core["rust-version"])
                dependency = package["dependencies"]["ddes"]
                self.assertEqual(dependency["package"], core["name"])
                self.assertEqual(((ROOT / manifest).parent / dependency["path"]).resolve(), ROOT / "ddes-rs")
                self.assertEqual(toml(toolchain)["toolchain"]["channel"], channel)
                lock = toml(str(Path(manifest).with_name("Cargo.lock")))
                locked_core = [p for p in lock["package"] if p["name"] == core["name"]]
                self.assertEqual(len(locked_core), 1)
                self.assertEqual(locked_core[0]["version"], core["version"])
                self.assertNotIn("source", locked_core[0])


if __name__ == "__main__":
    unittest.main()
