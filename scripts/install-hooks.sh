#!/usr/bin/env bash
# Reproducibly install repository-local Git hooks.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
git config --local core.hooksPath .githooks
chmod +x .githooks/pre-commit .githooks/pre-push scripts/validate.sh scripts/publish.sh
echo "Installed Git hooks from .githooks (core.hooksPath set locally)."
