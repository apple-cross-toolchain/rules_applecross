"""wrapped_clang off a Mac (APPLECROSS_TOOLCHAIN_BIN: LLVM's clang) with flags
only Xcode's clang accepts: refused, unless RULES_APPLECROSS_RELAX_APPLE_CLANG_FLAGS
(--config=relax_clang_apple_flags) has them dropped, values and all. The
wrapper's log-only mode prints the clang command instead of running it."""

import os
import subprocess
import sys
import unittest

WRAPPED_CLANG = os.path.abspath(sys.argv.pop(1))


def run(args, relax):
    tmp = os.environ["TEST_TMPDIR"]
    env = {
        "PATH": "/bin:/usr/bin",
        "DEVELOPER_DIR": tmp,
        "SDKROOT": tmp,
        "APPLECROSS_TOOLCHAIN_BIN": tmp,
        "__WRAPPED_CLANG_LOG_ONLY": "1",
    }
    if relax:
        env["RULES_APPLECROSS_RELAX_APPLE_CLANG_FLAGS"] = "1"
    return subprocess.run([WRAPPED_CLANG] + args, env=env, capture_output=True, text=True, cwd=tmp)


ARGS = [
    "-c", "main.m",
    "-fobjc-relative-method-lists",
    "-fbounds-safety-soft-traps=call",
    "-index-store-path", "index",
    "-fdepscan-prefix-map", "/old", "/new",
    "-O2",
]


class AppleFlagsTest(unittest.TestCase):
    def test_refused_by_default(self):
        result = run(ARGS, relax=False)
        self.assertNotEqual(0, result.returncode)
        for flag in ["-fobjc-relative-method-lists", "-fbounds-safety-soft-traps=call", "-index-store-path", "-fdepscan-prefix-map"]:
            self.assertIn("error: " + flag + " is a flag only Xcode's clang accepts", result.stderr)
        self.assertIn("--config=relax_clang_apple_flags", result.stderr)

    def test_dropped_when_relaxed(self):
        result = run(ARGS, relax=True)
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertIn("warning: dropping -fobjc-relative-method-lists", result.stderr)
        # The command (log-only mode prints its arguments, then a response
        # file's; here everything is in the response file).
        command = result.stdout.split()
        response = [a for a in command if a.startswith("@")]
        args = open(response[0][1:]).read().split() if response else command
        self.assertEqual(["-c", "main.m", "-O2"], [a for a in args if a in ("-c", "main.m", "-O2")])
        for gone in ["-fobjc-relative-method-lists", "-fbounds-safety-soft-traps=call", "-index-store-path", "index", "-fdepscan-prefix-map", "/old", "/new"]:
            self.assertNotIn(gone, args)

    def test_other_flags_untouched(self):
        result = run(["-c", "main.m", "-fobjc-arc", "-fno-stack-check"], relax=False)
        self.assertEqual(0, result.returncode, result.stderr)


if __name__ == "__main__":
    unittest.main()
