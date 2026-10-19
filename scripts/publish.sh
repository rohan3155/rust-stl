#!/usr/bin/env bash
# Repository-local publishing wrapper for rust-stl.
#
# Usage:
#   scripts/publish.sh [cargo publish arguments...]
#   scripts/publish.sh --publish [cargo publish arguments...]
#
# Without --publish this performs a dry run only. Passing --publish runs the
# real publication after the dry run succeeds.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

publish=false
args=()
for arg in "$@"; do
    if [[ "$arg" == "--publish" || "$arg" == "--release" ]]; then
        publish=true
    else
        args+=("$arg")
    fi
done

./scripts/validate.sh
cargo publish --dry-run --locked "${args[@]}"

if [[ "$publish" == "true" ]]; then
    cargo publish --locked "${args[@]}"
else
    echo "Dry run completed successfully. Re-run with --publish to publish for real."
fi
