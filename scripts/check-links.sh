#!/usr/bin/env bash
set -euo pipefail

# Simple link check using lychee (optional)
if ! command -v lychee >/dev/null 2>&1; then
  echo "Install lychee: cargo install lychee"
  exit 1
fi

lychee --no-progress --max-concurrency 8 --exclude-file .lycheeignore .
