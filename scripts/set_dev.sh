#!/usr/bin/env bash

set -euo pipefail

if [ $# -lt 1 ]; then
  echo "Použitie: $0 <source_branch>"
  exit 2
fi

SRC="$1"
TARGET="dev"
REMOTE="origin"
TS="$(date +'%y%m%d%H%M')"   # YYmmddHHMM
BACKUP="archive/${TARGET}/${TS}"

# fetch remote refs
git fetch "${REMOTE}"

# 1) najprv lokálne premenovať dev -> archive/dev/TS (ak existuje),
#    inak vytvoriť backup vetvu z origin/dev (fallback)
if git show-ref --verify --quiet "refs/heads/${TARGET}"; then
  git branch -m "${TARGET}" "${BACKUP}"
else
  # ak lokálne dev neexistuje, vytvor backup z remote (ak remote existuje)
  git branch "${BACKUP}" "${REMOTE}/${TARGET}" 2>/dev/null || true
fi

# 2) pushnúť backup na origin
git push "${REMOTE}" "${BACKUP}" 2>/dev/null || true

# 3) ak lokálna SRC vetva neexistuje, vytvor ju z origin/SRC (umožní push)
if ! git show-ref --verify --quiet "refs/heads/${SRC}"; then
  git branch "${SRC}" "${REMOTE}/${SRC}"
fi

# 4) prepísať remote dev obsahom SRC
git push "${REMOTE}" "${SRC}:${TARGET}" --force-with-lease

# 5) lokálne premenovať SRC -> dev (ak SRC existuje), inak vytvoriť dev z origin/SRC
git branch -m "${SRC}" "${TARGET}" 2>/dev/null || git branch -f "${TARGET}" "${REMOTE}/${SRC}"

# 6) nastaviť upstream pre novú lokálnu dev
git push -u "${REMOTE}" "${TARGET}" --force-with-lease

# 7) odstrániť remote SRC (nech nestraší) — ignoruj chyby
git push "${REMOTE}" --delete "${SRC}" 2>/dev/null || true

# 8) odstrániť lokálnu SRC ak ešte nebola premenovaná
git branch -D "${SRC}" 2>/dev/null || true

echo "Hotovo."
echo "Backup: ${REMOTE}/${BACKUP}"
echo "New dev: ${REMOTE}/${TARGET}"
