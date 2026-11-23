#!/usr/bin/env bash
# Regression tests for scripts/preflight-env.sh. Run: scripts/tests/test-preflight-env.sh
set -uo pipefail
SCRIPT="$(cd "$(dirname "$0")/.." && pwd)/preflight-env.sh"
fail=0
check() { if eval "$2"; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi; }

out="$(env -i PATH="$PATH" bash "$SCRIPT" smoke 2>&1)"; rc=$?
check "missing vars exit 1" '[ $rc -eq 1 ]'
check "missing names reported" 'grep -q "MISSING: CONTRACT_ID" <<<"$out"'

secret="SUPERSECRETVALUE123"
out="$(env -i PATH="$PATH" CONTRACT_ID="$secret" ADMIN_ACCOUNT="$secret" INSTRUCTOR_ACCOUNT="$secret" STUDENT_ACCOUNT="$secret" STELLAR_ACCOUNT="$secret" bash "$SCRIPT" all 2>&1)"; rc=$?
check "all set exits 0" '[ $rc -eq 0 ]'
check "values never printed" '! grep -q "$secret" <<<"$out"'

env -i PATH="$PATH" bash "$SCRIPT" bogus >/dev/null 2>&1; rc=$?
check "bad usage exits 64" '[ $rc -eq 64 ]'
exit $fail
