"""The library's generated header ends with the rewriter's marker: configure()'s
swift_generated_header_rewriter ran after the compile."""

import sys
import unittest

MARKER = "// Rewritten by //tests/swift_toolchain_attrs:header_rewriter.\n"
HEADER = sys.argv.pop(1)


class HeaderRewrittenTest(unittest.TestCase):
    def test_marker(self):
        with open(HEADER) as f:
            self.assertTrue(f.read().endswith(MARKER))


if __name__ == "__main__":
    unittest.main()
