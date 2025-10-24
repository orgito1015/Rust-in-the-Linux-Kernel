#!/usr/bin/env bash
set -euo pipefail

# Basic local setup
if ! command -v markdownlint >/dev/null 2>&1; then
  echo "Install markdownlint-cli (npm i -g markdownlint-cli) for local linting."
fi

echo "Setup complete."
