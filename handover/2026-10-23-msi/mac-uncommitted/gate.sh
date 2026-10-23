#!/usr/bin/env bash
# QicTrader fast gate. The ONLY local gate the ship pipeline runs.
# The full test suite runs once, in CI, on the ticket's single push.
#
#   .claude/scripts/gate.sh [--base <ref>] [--tests "<target> <target>"] [--no-frontend]
#
# Runs, for the backend: suppression greps, migration collision check,
# cargo check, cargo clippy --all-targets -D warnings, cargo test --lib, and
# every tests/<name>.rs that the diff touched or that names a changed src module.
# For the frontend (only when the diff touched it): typecheck, lint, build.
# Exit status is the gate verdict. Receipts go to .scratch/gate/<timestamp>/.
set -uo pipefail

ROOT="${QIC_GATE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
BASE="origin/main"; EXTRA_TESTS=""; DO_FE=1
while [ $# -gt 0 ]; do
  case "$1" in
    --base) BASE="$2"; shift 2 ;;
    --tests) EXTRA_TESTS="$2"; shift 2 ;;
    --no-frontend) DO_FE=0; shift ;;
    *) echo "gate: unknown option $1" >&2; exit 2 ;;
  esac
done

if [ "${QIC_GATE_NO_ENV:-0}" != "1" ] && [ -f "$HOME/.config/qic/.env" ]; then
  set -a; . "$HOME/.config/qic/.env"; set +a
fi
: "${DATABASE_URL:=postgres://postgres@127.0.0.1:5433/qictrader_test}"
export DATABASE_URL CARGO_INCREMENTAL=0 RUST_MIN_STACK=2147483648
ulimit -s unlimited 2>/dev/null || true

STAMP="$(date +%Y%m%dT%H%M%S)"; OUT="$ROOT/.scratch/gate/$STAMP"; mkdir -p "$OUT"
FAIL=0
step() { # step <name> <cmd...>; logs to $OUT/<name>.log, prints one verdict line
  local name="$1"; shift; local t0=$SECONDS
  if "$@" >"$OUT/$name.log" 2>&1; then echo "  ok   $name ($((SECONDS-t0))s)"
  else echo "  FAIL $name ($((SECONDS-t0))s) -> $OUT/$name.log"; tail -20 "$OUT/$name.log" | sed 's/^/       /'; FAIL=1; fi
}
changed() { # changed <repo dir>: committed since merge-base with BASE, plus uncommitted
  ( cd "$1" && { git diff --name-only "$(git merge-base "$BASE" HEAD 2>/dev/null || echo HEAD)" HEAD; git diff --name-only HEAD; git ls-files --others --exclude-standard; } | sort -u )
}

echo "gate --fast · base $BASE · receipts $OUT"

BE="$ROOT/qictrader-backend-rs"
if [ -d "$BE/src" ]; then
  BE_CHANGED="$(changed "$BE")"; printf '%s\n' "$BE_CHANGED" > "$OUT/backend-changed.txt"
  if [ -n "$BE_CHANGED" ]; then
    echo "backend: $(printf '%s\n' "$BE_CHANGED" | grep -c .) changed files"
    cd "$BE"
    MB="$(git merge-base "$BASE" HEAD 2>/dev/null || echo HEAD)"
    added_lines() { { git diff "$MB" -- src/; git diff HEAD -- src/; git ls-files --others --exclude-standard -z -- src/ | xargs -0 -r sed 's/^/+/'; } | grep -E '^\+' | grep -v -E '^\+\+\+'; }
    echo "backend: pre-existing suppressions on base (not gated): let _ = $(grep -rn 'let _ =' src/ | grep -v '#\[cfg(test)\]' | wc -l | tr -d ' '), .ok(); $(grep -rn '\.ok();' src/ | grep -v test | wc -l | tr -d ' ')"
    step suppression-added-let-underscore bash -c "MB='$MB'; $(declare -f added_lines); ! added_lines | grep -E 'let _ = ' | grep ."
    step suppression-added-ok-discard     bash -c "MB='$MB'; $(declare -f added_lines); ! added_lines | grep -E '\.ok\(\);' | grep ."
    step migration-collisions     bash -c '! ls migrations/ | sed -E "s/\.(up|down)\.sql$//" | sort -u | awk -F_ "{print \$1}" | sort | uniq -d | grep .'
    step cargo-check  cargo check --all-targets
    step cargo-clippy cargo clippy --all-targets -- -D warnings
    step cargo-test-lib cargo test --lib
    STEMS="$(printf '%s\n' "$BE_CHANGED" | grep -E '^src/.*\.rs$' | xargs -n1 basename 2>/dev/null | sed 's/\.rs$//' | grep -v -E '^(mod|lib|main)$' | sort -u)"
    TARGETS="$(printf '%s\n' "$BE_CHANGED" | grep -E '^tests/[^/]+\.rs$' | xargs -n1 basename 2>/dev/null | sed 's/\.rs$//')"
    if [ -n "$STEMS" ]; then
      for f in tests/*.rs; do
        for s in $STEMS; do
          if grep -q -E "::$s\\b|\\b$s::" "$f" || [[ "$(basename "$f")" == *"$s"* ]]; then TARGETS="$TARGETS $(basename "$f" .rs)"; break; fi
        done
      done
    fi
    TARGETS="$(printf '%s\n' $TARGETS $EXTRA_TESTS | sort -u | grep .)"
    printf '%s\n' "$TARGETS" > "$OUT/backend-test-targets.txt"
    if [ -n "$TARGETS" ]; then
      echo "backend: integration targets: $(echo $TARGETS | wc -w | tr -d ' ')"
      PGBIN="$(dirname "$(pgrep -fl 'bin/postgres ' | grep -o '/[^ ]*/bin/postgres' | head -1)" 2>/dev/null)"
      case "$DATABASE_URL" in
        *@127.0.0.1*|*@localhost*)
          if [ -n "$PGBIN" ] && [ -x "$PGBIN/dropdb" ]; then
            DBN="${DATABASE_URL##*/}"; DBH="${DATABASE_URL#*@}"; DBH="${DBH%%/*}"; DBP="${DBH##*:}"; DBH="${DBH%%:*}"
            step fresh-test-db bash -c "\"$PGBIN/dropdb\" -h $DBH -p $DBP -U postgres --if-exists $DBN && \"$PGBIN/createdb\" -h $DBH -p $DBP -U postgres $DBN"
          else
            echo "  note fresh-test-db skipped (no local dropdb found); leftover rows can fail tests that CI passes"
          fi ;;
      esac
      ARGS=""; for t in $TARGETS; do ARGS="$ARGS --test $t"; done
      PGBIN="$(dirname "$(ps -axo command | grep -m1 '[p]ostgres -D' | awk '{print $1}')" 2>/dev/null)"
      DBNAME="${DATABASE_URL##*/}"; DBHOSTPORT="$(printf '%s' "$DATABASE_URL" | sed -E 's#.*@([^/]+)/.*#\1#')"
      if [ -x "$PGBIN/dropdb" ]; then
        step fresh-test-db bash -c "'$PGBIN/dropdb' --if-exists -h '${DBHOSTPORT%%:*}' -p '${DBHOSTPORT##*:}' -U postgres '$DBNAME' && '$PGBIN/createdb' -h '${DBHOSTPORT%%:*}' -p '${DBHOSTPORT##*:}' -U postgres '$DBNAME'"
      else
        echo "  warn fresh-test-db: no local dropdb found, reusing $DBNAME (stale fixture rows can fail tests that CI passes)"
      fi
      run_targets() {
        local t host="${DBHOSTPORT%%:*}" port="${DBHOSTPORT##*:}" tpl="qic_631_ci_gate_$(date +%s)_tpl" base="${DATABASE_URL%/*}"
        if [ -x "$PGBIN/dropdb" ] && [ -f scripts/test-db-bootstrap/Cargo.toml ]; then
          "$PGBIN/createdb" -h "$host" -p "$port" -U postgres -T template0 "$tpl" || return 1
          REQUIRE_TEST_DB=1 DATABASE_URL="$base/$tpl" EXPECTED_TEST_DATABASE="$tpl" cargo run --locked --manifest-path scripts/test-db-bootstrap/Cargo.toml || { "$PGBIN/dropdb" -h "$host" -p "$port" -U postgres --if-exists "$tpl"; return 1; }
          "$PGBIN/psql" -h "$host" -p "$port" -U postgres -d postgres -qc "ALTER DATABASE $tpl ALLOW_CONNECTIONS false" || return 1
          for t in $TARGETS; do
            "$PGBIN/dropdb" --if-exists -h "$host" -p "$port" -U postgres "$DBNAME" \
              && "$PGBIN/createdb" -h "$host" -p "$port" -U postgres -T "$tpl" "$DBNAME" || return 1
            REQUIRE_TEST_DB=1 cargo test --test "$t" -- --test-threads=1 || { "$PGBIN/dropdb" -h "$host" -p "$port" -U postgres --if-exists "$tpl"; return 1; }
          done
          "$PGBIN/dropdb" -h "$host" -p "$port" -U postgres --if-exists "$tpl"
        else
          for t in $TARGETS; do REQUIRE_TEST_DB=1 cargo test --test "$t" || return 1; done
        fi
      }
      step cargo-test-targets run_targets
    else
      echo "backend: no integration target matches the diff"
    fi
  else
    echo "backend: unchanged, skipped"
  fi
fi

FE="$ROOT/frontend"
if [ "$DO_FE" = 1 ] && [ -f "$FE/package.json" ]; then
  FE_CHANGED="$(changed "$FE")"; printf '%s\n' "$FE_CHANGED" > "$OUT/frontend-changed.txt"
  if [ -n "$FE_CHANGED" ]; then
    echo "frontend: $(printf '%s\n' "$FE_CHANGED" | grep -c .) changed files"
    cd "$FE"; export NODE_OPTIONS=--max-old-space-size=8192
    step bun-typecheck bun run typecheck
    frontend_lint_no_new_errors() {
      local mb f new old status=0 files
      mb="$(git merge-base "$BASE" HEAD 2>/dev/null || echo HEAD)"
      files="$(printf '%s\n' "$FE_CHANGED" | grep -E '\.(ts|tsx|js|jsx|mjs)$' | while read -r f; do [ -f "$f" ] && printf '%s\n' "$f"; done)"
      [ -z "$files" ] && { echo "no changed lintable frontend files"; return 0; }
      while read -r f; do
        [ -z "$f" ] && continue
        new="$(bunx eslint -f json "$f" 2>/dev/null | python3 -c 'import sys,json; d=json.load(sys.stdin); print(sum(x["errorCount"] for x in d))' 2>/dev/null)"
        [ -z "$new" ] && new=ERR
        if git cat-file -e "$mb:$f" 2>/dev/null; then
          old="$(git show "$mb:$f" | bunx eslint --stdin --stdin-filename "$f" -f json 2>/dev/null | python3 -c 'import sys,json; d=json.load(sys.stdin); print(sum(x["errorCount"] for x in d))' 2>/dev/null)"
          [ -z "$old" ] && old=ERR
        else
          old=0
        fi
        if [ "$new" = "ERR" ] || [ "$old" = "ERR" ]; then echo "FAIL $f: eslint did not produce a result (new=$new base=$old)"; status=1
        elif [ "$new" -gt "$old" ]; then echo "FAIL $f: errors rose from $old to $new"; status=1
        else echo "ok   $f: errors $old -> $new"; fi
      done <<< "$files"
      return $status
    }
    step bun-lint-no-new-errors frontend_lint_no_new_errors
    step bun-build     bun run build
  else
    echo "frontend: unchanged, skipped"
  fi
fi

if [ "$FAIL" = 0 ]; then echo "GATE PASS · $OUT"; else echo "GATE FAIL · $OUT"; fi
exit $FAIL
