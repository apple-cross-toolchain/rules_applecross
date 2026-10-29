#!/bin/bash
# appintentsnltrainingprocessor off a Mac. App Shortcuts Flexible Matching
# trains Apple's language model on an app's shortcut phrases, which has no
# Linux version. Strict (the default), the build fails, saying how to turn it
# off; relaxed (--config=relax_app_intents_flexible_matching,
# RULES_APPLECROSS_RELAX_APP_INTENTS_FLEXIBLE_MATCHING=1, or all of
# --config=relaxed), its assets are left out, as with flexible matching off.

if [[ "${RULES_APPLECROSS_RELAX_APP_INTENTS_FLEXIBLE_MATCHING:-}" != "1" && "${RULES_APPLECROSS_RESOURCE_OUTPUT_MODE:-}" != "relaxed" ]]; then
  echo "error: appintentsnltrainingprocessor: App Shortcuts Flexible Matching can't be trained off a Mac; set app_intents_flexible_matching = False on the bundle, or build with --config=relax_app_intents_flexible_matching to leave its assets out" >&2
  exit 1
fi
echo "warning: appintentsnltrainingprocessor: relaxed: leaving App Shortcuts Flexible Matching's assets out" >&2
