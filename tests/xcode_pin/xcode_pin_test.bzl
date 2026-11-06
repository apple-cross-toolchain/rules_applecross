"""check_xcode_pin: the pinned build, another build of the version with and
without relax_xcode_build, and another version."""

load("@bazel_skylib//lib:unittest.bzl", "asserts", "unittest")
load("//toolchain:xcode_pin.bzl", "check_xcode_pin")

_PINNED = struct(build = "27B52", swift_version = "6.4.2")

def _check(xcode_version, xcode_build, swift, relax):
    return check_xcode_pin("tc", "27.2", _PINNED, xcode_version, xcode_build, swift, relax)

def _pinned_build_test(ctx):
    env = unittest.begin(ctx)
    asserts.equals(env, (None, None), _check("27.2", "27B52", "6.4.2", False))
    return unittest.end(env)

def _other_build_test(ctx):
    env = unittest.begin(ctx)
    error, warning = _check("27.2", "27B48", "6.4.2", False)
    asserts.true(env, error and "--config=relax_xcode_build" in error, error)
    asserts.equals(env, None, warning)
    return unittest.end(env)

def _other_build_relaxed_test(ctx):
    env = unittest.begin(ctx)
    error, warning = _check("27.2", "27B48", "6.4.2", True)
    asserts.equals(env, None, error)
    asserts.true(env, warning and "27B48" in warning and "matched to 27B52's" in warning, warning)
    return unittest.end(env)

def _other_build_other_swift_test(ctx):
    env = unittest.begin(ctx)
    error, _ = _check("27.2", "27B48", "6.4.3", True)
    asserts.true(env, error and "built by Swift 6.4.3, not 6.4.2" in error, error)
    return unittest.end(env)

def _other_version_test(ctx):
    env = unittest.begin(ctx)
    error, _ = _check("27.0", "27A266a", "6.4.0", True)
    asserts.true(env, error and "are from Xcode 27.0 (27A266a)" in error, error)
    return unittest.end(env)

pinned_build_test = unittest.make(_pinned_build_test)
other_build_test = unittest.make(_other_build_test)
other_build_relaxed_test = unittest.make(_other_build_relaxed_test)
other_build_other_swift_test = unittest.make(_other_build_other_swift_test)
other_version_test = unittest.make(_other_version_test)

def xcode_pin_test_suite(name):
    unittest.suite(
        name,
        pinned_build_test,
        other_build_test,
        other_build_relaxed_test,
        other_build_other_swift_test,
        other_version_test,
    )
