#!/usr/bin/env bash
set -euo pipefail

if [[ "${RUN_LOCALNET_SMOKE:-0}" != "1" ]]; then
  echo "Skipping localnet smoke test. Set RUN_LOCALNET_SMOKE=1 in a supported Unix/Linux scaffold environment."
  exit 0
fi

if ! command -v logos-scaffold >/dev/null 2>&1 && ! command -v lgs >/dev/null 2>&1; then
  echo "logos-scaffold/lgs is not available; skipping optional localnet smoke test."
  exit 0
fi

echo "Localnet smoke test placeholder: copy samples/spel-gated-config-program into a scaffold project, then run setup/localnet/deploy/tx."
echo "Maintainers should replace this placeholder once the official SPEL/LEE dependency pins and scaffold flow are confirmed."
