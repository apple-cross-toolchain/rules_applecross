"""The Swift binary built for Linux runs."""

import subprocess
import sys
import unittest

TOOL = sys.argv.pop(1)


class LinuxToolTest(unittest.TestCase):
    def test_runs(self):
        out = subprocess.run([TOOL], capture_output=True, text=True, check=True).stdout
        self.assertEqual("Hello from Swift on Linux\n", out)


if __name__ == "__main__":
    unittest.main()
