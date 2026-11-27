#!/usr/bin/env bash
# Deployment preflight: report which required environment variables are missing.
#
# Usage: scripts/preflight-env.sh [deploy|smoke|all]   (default: all)
#
# Only variable NAMES are printed, never their values, so the output is safe to
# paste into an issue or CI log. See docs/deployment-preflight.md.
#
# Exit codes: 0 all required variables set, 1 one or more missing, 64 bad usage.

set -euo pipefail

DEPLOY_VARS=(STELLAR_ACCOUNT)
SMOKE_VARS=(CONTRACT_ID ADMIN_ACCOUNT INSTRUCTOR_ACCOUNT STUDENT_ACCOUNT)

case "${1:-all}" in
  deploy) required=("${DEPLOY_VARS[@]}") ;;
  smoke)  required=("${SMOKE_VARS[@]}") ;;
  all)    required=("${DEPLOY_VARS[@]}" "${SMOKE_VARS[@]}") ;;
  *) echo "usage: $0 [deploy|smoke|all]" >&2; exit 64 ;;
esac

missing=0
for var in "${required[@]}"; do
  if [ -z "${!var:-}" ]; then
    echo "MISSING: $var"
    missing=1
  else
    echo "OK:      $var"
  fi
done

if [ "$missing" -ne 0 ]; then
  echo "preflight failed: set the MISSING variables above (see docs/deployment-preflight.md)" >&2
  exit 1
fi
echo "preflight passed"
