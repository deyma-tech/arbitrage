#!/usr/bin/env bash

set -euo pipefail

git fetch origin
git checkout dev
git reset --hard origin/dev
git clean -fd
git pull origin dev
