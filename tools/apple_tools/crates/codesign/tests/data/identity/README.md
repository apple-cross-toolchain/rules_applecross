Fixtures for `identity_signature.rs`, from one signing by Xcode 27.0's
`codesign` on macOS 27.0.1 (a leased VM), of rules_applecross's
`examples/ios/CleanRoom` app:

    codesign -v --sign <identity> --force --entitlements <entitlements> \
      --generate-entitlement-der --timestamp=none CleanRoom.app

- `test-key.pem`, `test-chain.pem`: a throwaway RSA key and its certificate
  chain (leaf "RAC Test Developer (TEAMID1234)", root "RAC Test CA"), made
  with `openssl` for these tests only. They sign nothing anywhere else and are
  trusted nowhere.
- `apple-cd.bin`: the code directory codesign wrote (slot 0).
- `apple-requirements.bin`: its requirement set (slot 2), the designated
  requirement `identifier "com.example.CleanRoom" and certificate root = H"…"`.
- `apple-cms.der`: its CMS signature (slot 0x10000), signed at
  2026-10-27T05:46:13Z (Unix time 1793079973).
