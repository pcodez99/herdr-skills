import json
import os
from pathlib import Path
import struct
import subprocess
import unittest
from zipfile import ZipFile


ROOT = Path(__file__).resolve().parents[1]


class Packages(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        subprocess.run(["make", "package"], cwd=ROOT, check=True, capture_output=True)

    def test_self_contained_packages(self):
        required = {"LICENSE", "README.md", "PRIVACY.md", "docs/PUBLISHING.md", "assets/icon.png",
                    "skills/herdr-team/SKILL.md", "skills/herdr-team/TEAMMATE.md",
                    "skills/herdr-team/LIFECYCLE.md", "skills/herdr-team/TROUBLESHOOTING.md"}
        for host, manifest_path in [("codex", "plugin.json"), ("claude", ".claude-plugin/plugin.json")]:
            with self.subTest(host=host), ZipFile(ROOT / "dist" / f"herdr-{host}.zip") as archive:
                self.assertIsNone(archive.testzip())
                names = set(archive.namelist())
                self.assertTrue(required | {manifest_path} <= names)
                self.assertTrue(all(name.startswith(("skills/herdr-team/", "assets/", "hooks/"))
                                    or name in required | {manifest_path} for name in names))
                manifest = json.loads(archive.read(manifest_path))
                self.assertEqual(manifest["author"]["name"], "pcodez")
                self.assertEqual(manifest["license"], "MIT")
                self.assertEqual(manifest["version"], "0.6.0")
                icon = archive.read("assets/icon.png")
                self.assertEqual(icon[:8], b"\x89PNG\r\n\x1a\n")
                width, height = struct.unpack(">II", icon[16:24])
                self.assertEqual(width, height)
                self.assertTrue(512 <= width <= 2048)
                self.assertLess(len(icon), 2 * 1024 * 1024)
                if host == "codex":
                    extension = manifest["extensions"]["com.openai"]
                    self.assertTrue(extension["requires_local_executor"])
                    self.assertNotIn("hooks", extension)
                    self.assertFalse(any(name.startswith("hooks/") for name in names))
                    interface = extension["interface"]
                    self.assertLessEqual(len(interface["shortDescription"]), 30)
                    for field in ("logo", "composerIcon"):
                        self.assertIn(interface[field].removeprefix("./"), names)
                else:
                    self.assertIn("hooks/hooks.json", names)
                    self.assertIn(manifest["icon"].removeprefix("./"), names)

    def test_hook_inside_and_outside_herdr(self):
        hook = ROOT / "hooks/activate.sh"
        subprocess.run(["sh", "-n", str(hook)], check=True)
        env = dict(os.environ)
        env.pop("HC_TERMINAL_ID", None)
        self.assertEqual(subprocess.check_output(["sh", str(hook)], env=env), b"")
        env["HC_TERMINAL_ID"] = "t-test"
        self.assertIn(b"address t-test.", subprocess.check_output(["sh", str(hook)], env=env))


if __name__ == "__main__":
    unittest.main()
