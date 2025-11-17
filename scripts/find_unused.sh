#!/usr/bin/env bash
set -euo pipefail

# Použitie:
#   scripts/find_unused.sh [workspace-root] [--no-tests]
#   (predpoklad: máš nainštalované jq a ripgrep)

ROOT="${1:-.}"
INCLUDE_TESTS=1
if [[ "${2-}" == "--no-tests" ]]; then
  INCLUDE_TESTS=0
fi

cd "$ROOT"

# Overíme, že sme vo workspace (alebo dáme lepšie hlášku)
if ! cargo metadata -q >/dev/null 2>&1; then
  echo "Chyba: Neviem spustiť 'cargo metadata' v $PWD. Si v koreňovom priečinku workspace-u?"
  exit 1
fi

# Zoznam workspace balíkov (názvy) – robustná verzia
readarray -t PACKAGES < <(
  cargo metadata --no-deps --format-version 1 \
  | jq -r '
      .workspace_members[] as $wid
      | .packages[]
      | select(.id == $wid)
      | .name
    ' \
  | sort -u
)

if ((${#PACKAGES[@]} == 0)); then
  echo "Chyba: Workspace nemá žiadne packages, alebo sa metadata nepodarilo prečítať."
  echo "Tip: ak spúšťaš z pod-crate, skús:  scripts/find_unused.sh /cesta/na/korene/workspace"
  exit 1
fi

# Ignorované cesty pre hľadanie použitia
IGNORE_GLOBS=(
  '!target/**'
  '!examples/**'
)
if [[ "$INCLUDE_TESTS" != "1" ]]; then
  IGNORE_GLOBS+=('!tests/**' '!**/tests/**')
fi

declare -A TYPE_DEF_LINES   # "crate::path::Type" -> "file:line"
declare -A TYPE_BASENAMES   # "crate::path::Type" -> "Type"

echo "→ Zbieram public typy z ${#PACKAGES[@]} crates…"
for pkg in "${PACKAGES[@]}"; do
  # lib
  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    if [[ "$line" =~ ^pub[[:space:]]+(struct|enum|type|trait|union)[[:space:]]+(.+)$ ]]; then
      full="${BASH_REMATCH[2]}"
      base="${full##*::}"
      defloc=$(rg -n --type rust ${IGNORE_GLOBS[@]/#/-g } -w \
        -e "^\s*pub(\([^)]*\))?\s+(struct|enum|type|trait|union)\s+$base\b" \
        | head -n1 || true)
      TYPE_DEF_LINES["$full"]="${defloc:-unknown}"
      TYPE_BASENAMES["$full"]="$base"
    fi
  done < <(cargo public-items -p "$pkg" --lib --no-deps 2>/dev/null || true)

  # bins (ak sú)
  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    if [[ "$line" =~ ^pub[[:space:]]+(struct|enum|type|trait|union)[[:space:]]+(.+)$ ]]; then
      full="${BASH_REMATCH[2]}"
      base="${full##*::}"
      defloc=$(rg -n --type rust ${IGNORE_GLOBS[@]/#/-g } -w \
        -e "^\s*pub(\([^)]*\))?\s+(struct|enum|type|trait|union)\s+$base\b" \
        | head -n1 || true)
      TYPE_DEF_LINES["$full"]="${TYPE_DEF_LINES[$full]:-${defloc:-unknown}}"
      TYPE_BASENAMES["$full"]="$base"
    fi
  done < <(cargo public-items -p "$pkg" --bins --no-deps 2>/dev/null || true)
done

echo "→ Hľadám nepoužité public typy (ignorujem definitions; ignorujem examples/ $( [[ $INCLUDE_TESTS != 1 ]] && echo ', tests/' ))…"
unused=()

for full in "${!TYPE_BASENAMES[@]}"; do
  base="${TYPE_BASENAMES[$full]}"

  # Použitie mena typu kde to nie je samotná definícia
  if ! rg --type rust ${IGNORE_GLOBS[@]/#/-g } -n -w "$base" \
        | rg -v -w "^\s*pub(\([^)]*\))?\s+(struct|enum|type|trait|union)\s+$base\b" \
        >/dev/null; then
    unused+=("$full")
  fi
done

if ((${#unused[@]} == 0)); then
  echo "✔ Nenašli sa žiadne zjavne nepoužívané public typy."
else
  echo "⚠ Pravdepodobne nepoužívané public typy:"
  for full in "${unused[@]}"; do
    echo " - $full   (${TYPE_DEF_LINES[$full]})"
  done
  echo
  echo "Poznámky:"
  echo "• Heuristika – ak sa typ používa len cez makro/proc-macro/generovaný kód, môže byť false-positive."
  echo "• Re-exporty pod iným menom môžu skresľovať. Ak používaš aliasy, zvaž zváženie prísnejších regexov."
fi
