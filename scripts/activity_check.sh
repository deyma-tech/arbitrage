#!/bin/bash
set -uo pipefail

cd /home/ubuntu
#cd /home/oxy/work/github/jozef-pridavok

GAC=./github-activity-check/target/release/github-activity-check
NAC=./npm-activity-check/target/release/npm-activity-check
HISTORY=/tmp

VERBOSE=0
TEST_MESSAGE=0

if [[ "${1-}" == "--verbose" || "${1-}" == "-v" ]]; then
  VERBOSE=1
  shift
fi

while [[ $# -gt 0 ]]; do
  case $1 in
    --verbose|-v)
      VERBOSE=1
      shift
      ;;
    --test|-t)
      TEST_MESSAGE=1
      shift
      ;;
    *)
      shift
      ;;
  esac
done

verbose() {
  if (( VERBOSE )); then
    echo "[VERBOSE] $*"
  fi
}

tg_send() {
  local msg="$1"
  if (( VERBOSE )); then
    verbose "TG message: $msg"
    return 0
  fi
  curl -sS -X POST "https://api.telegram.org/bot${TELEGRAM_TOKEN}/sendMessage" \
    -d "chat_id=${TELEGRAM_CHAT_ID_ARB_BOT_ACTIVITY_CHECK}" \
    -d "parse_mode=HTML" \
    --data-urlencode "text=${msg}" \
    -d "disable_web_page_preview=true" \
    -d "disable_notification=true" > /dev/null
}

if (( TEST_MESSAGE )); then
  verbose "Sending test message to Telegram..."
  tg_send "Test message"
  verbose "Test message sent."
  exit 0
fi

check_gac_total_commits() {
  local owner="$1"
  local repo="$2"

  "$GAC" "$owner" "$repo" --history "$HISTORY/$owner-$repo.json" --check commits_total
  local rc=$?

  if [[ $rc -ne 0 ]]; then
    tg_send "Repo <b>$owner/$repo</b> has NEW commit(s): $rc"
  else
    verbose "Repo $owner/$repo doesn't have new commits"
  fi
}

check_npm_new_version() {
  local owner="$1"
  local repo="$2"

  "$NAC" "$owner/$repo" --history "$HISTORY/$owner-$repo.json" --check latest_version
  local rc=$?

  if [[ $rc -ne 0 ]]; then
    tg_send "Repo <b>$owner/$repo</b> has NEW version(s): $rc"
  else
    verbose "Repo $owner/$repo doesn't have new versions"
  fi
}

# TODO: obric, orca swap v2

repos=(
  "raydium-io raydium-clmm"
  "raydium-io raydium-amm"
  "raydium-io raydium-cp-swap"
  "MeteoraAg damm-v2"
  "MeteoraAg dlmm-sdk"
  "orca-so whirlpools"
  "DefiTuna fusionamm-sdk"
  "GooseFX1 gamma-swap"
  "stabbleorg amm-sdk"
  "saros-xyz saros-dlmm-sdk-rs"
)

for entry in "${repos[@]}"; do
  set -- $entry
  check_gac_total_commits "$1" "$2"
done

repos=(
  "@pump-fun pump-swap-sdk"
  "@lifinity sdk-v2"
  "@perena numeraire-sdk"
)

for entry in "${repos[@]}"; do
  set -- $entry
  check_npm_new_version "$1" "$2"
done

#cd /home/oxy/work/krypto/arbitrage/scripts
