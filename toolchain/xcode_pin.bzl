"""Whether a staged Xcode satisfies apple_cross_toolchain.configure()'s
xcode_version pin."""

def check_xcode_pin(repo, wanted_version, pinned, xcode_version, xcode_build, sdk_swift_version, relax_build):
    """Checks a staged Xcode against its pin.

    The pinned build passes. Another build of the same version passes with
    relax_build (RULES_APPLECROSS_RELAX_XCODE_BUILD=1, --config=relax_xcode_build)
    when its SDKs were built by the Swift the pin selects; the bundling
    tools' outputs are then still matched to the pinned build's.

    Args:
        repo: the toolchain repository's name, for messages.
        wanted_version: configure()'s xcode_version.
        pinned: its XCODE_VERSIONS entry (build, swift_version).
        xcode_version: the staged Xcode's version.
        xcode_build: the staged Xcode's build.
        sdk_swift_version: the Swift version that built the staged SDKs.
        relax_build: whether another build of the version is accepted.

    Returns:
        (error, warning): an error message if the staged Xcode doesn't
        satisfy the pin, else None; a warning to print, or None.
    """
    if xcode_version == wanted_version and xcode_build == pinned.build:
        return None, None
    if xcode_version == wanted_version and relax_build:
        if sdk_swift_version != pinned.swift_version:
            return ("Xcode {} ({}) staged for @{} has SDKs built by Swift {}, not {} as Xcode {} ({}) has: " +
                    "its modules don't load in the Swift xcode_version selects.").format(
                xcode_version,
                xcode_build,
                repo,
                sdk_swift_version or "(unknown)",
                pinned.swift_version,
                wanted_version,
                pinned.build,
            ), None
        return None, ("Xcode {} ({}) staged for @{} rather than the pinned build {}: its SDKs' Swift {} " +
                      "matches; the bundling tools' outputs are matched to {}'s (--config=relax_xcode_build).").format(
            xcode_version,
            xcode_build,
            repo,
            pinned.build,
            sdk_swift_version,
            pinned.build,
        )
    return ("xcode_version is {} ({}), but the Apple SDKs staged for @{} are from Xcode {} ({}). " +
            "On a Mac with another Xcode, --repo_env=RULES_APPLECROSS_LOCAL_XCODE=0 " +
            "stages the pinned one instead; another build of the same version, whose SDKs the " +
            "same Swift built, is accepted with --config=relax_xcode_build " +
            "(--repo_env=RULES_APPLECROSS_RELAX_XCODE_BUILD=1).").format(
        wanted_version,
        pinned.build,
        repo,
        xcode_version,
        xcode_build,
    ), None
