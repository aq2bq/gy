#!/usr/bin/env bash
# Physical-design gate (D-76, AC-54). Measures a crate's src/ and tests/:
#   1 file length  2 function length  3 glob imports  4 string-keyed attributes
#   5 test length  6 diff size       7 layer direction
# Usage: scripts/measure.sh <src-dir> <tests-dir> [--base <ref>] [--json]
# Exit code: 0 when nothing violates the limits, 1 otherwise.
set -u

FILE_MAX=300
FN_MAX=40
GLOB_MAX=0
KEY_MAX=1
TEST_MAX=300
DIFF_MAX=600

src=""; tests=""; base="HEAD"; json=0
while [ $# -gt 0 ]; do
  case "$1" in
    --base) base="${2:-HEAD}"; shift 2 ;;
    --json) json=1; shift ;;
    -h|--help) sed -n '2,6p' "$0"; exit 0 ;;
    *) if [ -z "$src" ]; then src="$1"; else tests="$1"; fi; shift ;;
  esac
done
[ -n "$src" ] || src="crates/gy-ledger/src"
[ -n "$tests" ] || tests="crates/gy-ledger/tests"

tmp=$(mktemp -d) || exit 1
trap 'rm -rf "$tmp"' EXIT
: > "$tmp/over_files"; : > "$tmp/fns"; : > "$tmp/fn_over"; : > "$tmp/globs"
: > "$tmp/keys"; : > "$tmp/test_over"; : > "$tmp/layer_viol"

rs_files=$(find "$src" -name '*.rs' 2>/dev/null | sort)

# 1. files
files_total=0; files_max=0
for f in $rs_files; do
  n=$(wc -l < "$f" | tr -d ' ')
  files_total=$((files_total + 1))
  [ "$n" -gt "$files_max" ] && files_max=$n
  [ "$n" -gt "$FILE_MAX" ] && printf '%s:%s\n' "$f" "$n" >> "$tmp/over_files"
done

# 2. functions: from the `fn` line to the matching closing brace, blanks included
for f in $rs_files; do
  awk -v file="$f" '
    /^[[:space:]]*(pub(\([a-z]+\))?[[:space:]]+)?(const[[:space:]]+)?(async[[:space:]]+)?fn[[:space:]]/ {
      start=NR; depth=0; opened=0; infn=1
    }
    infn {
      for (i = 1; i <= length($0); i++) {
        c = substr($0, i, 1)
        if (c == "{") { depth++; opened=1 } else if (c == "}") { depth-- }
      }
      if (opened && depth <= 0) { printf "%s:%d:%d\n", file, start, NR - start + 1; infn=0 }
      else if (!opened && $0 ~ /;[[:space:]]*$/) { infn=0 }
    }
  ' "$f"
done > "$tmp/fns"
fn_total=$(wc -l < "$tmp/fns" | tr -d ' ')
awk -F: -v max="$FN_MAX" '$3 > max' "$tmp/fns" >> "$tmp/fn_over"

# 3. glob imports of the crate
grep -rnE '^[[:space:]]*use[[:space:]]+crate::[^;]*\*' $rs_files > "$tmp/globs" 2>/dev/null || true
glob_count=$(wc -l < "$tmp/globs" | tr -d ' ')

# 4. string-keyed attribute access in src/. A subscript counts only when it
# follows an identifier or a closing bracket, never an array literal.
grep -rnE '\.(get|insert|remove|contains_key)[[:space:]]*\([[:space:]]*"|[A-Za-z0-9_)\]]\[[[:space:]]*"' \
  $rs_files > "$tmp/keys" 2>/dev/null || true
key_count=$(wc -l < "$tmp/keys" | tr -d ' ')

# 5. test files
tests_total=0
for f in $(find "$tests" -maxdepth 1 -name '*.rs' 2>/dev/null | sort); do
  n=$(wc -l < "$f" | tr -d ' ')
  tests_total=$((tests_total + 1))
  [ "$n" -gt "$TEST_MAX" ] && printf '%s:%s\n' "$f" "$n" >> "$tmp/test_over"
done

# 6. diff against --base, excluding Cargo.lock. The score is the additions to
# kept files plus every line of a new file (tracked or still untracked).
numstat=$(git diff --numstat --no-renames "$base" 2>/dev/null | grep -v 'Cargo.lock' || true)
added=$(printf '%s\n' "$numstat" | awk '$1 ~ /^[0-9]+$/ { s += $1 } END { print s + 0 }')
removed=$(printf '%s\n' "$numstat" | awk '$2 ~ /^[0-9]+$/ { s += $2 } END { print s + 0 }')
untracked=0
for f in $(git status --porcelain --untracked-files=all 2>/dev/null | awk '$1 == "??" { print $2 }'); do
  case "$f" in *Cargo.lock) continue ;; esac
  n=$(wc -l < "$f" 2>/dev/null | tr -d ' ')
  untracked=$((untracked + ${n:-0}))
done
score=$((added + untracked))

# 7. layer direction among src/{store,model,ops,views}.rs or src/<layer>/**/*.rs
layers_present=0
for layer in store model ops views; do
  layer_files=$(
    {
      [ -f "$src/$layer.rs" ] && echo "$src/$layer.rs"
      [ -d "$src/$layer" ] && find "$src/$layer" -name '*.rs'
    } 2>/dev/null | sort
  )
  [ -n "$layer_files" ] || continue
  layers_present=1
  case "$layer" in
    store) allowed=" " ;;
    model) allowed=" store " ;;
    ops)   allowed=" store model " ;;
    views) allowed=" model ops " ;;
  esac
  for f in $layer_files; do
    while IFS= read -r entry; do
      for ref in $(printf '%s' "$entry" | grep -oE '(crate|super)::(store|model|ops|views)' | sed 's/.*:://' | sort -u); do
        case "$allowed" in
          *" $ref "*) ;;
          *) printf '%s:%s\n' "$f" "$entry" >> "$tmp/layer_viol" ;;
        esac
      done
    done < <(grep -nE '^[[:space:]]*use ' "$f" 2>/dev/null || true)
  done
done
layer_count=$(wc -l < "$tmp/layer_viol" | tr -d ' ')

over_file_count=$(wc -l < "$tmp/over_files" | tr -d ' ')
fn_over_count=$(wc -l < "$tmp/fn_over" | tr -d ' ')
test_over_count=$(wc -l < "$tmp/test_over" | tr -d ' ')
viol=$((over_file_count + fn_over_count + test_over_count + layer_count))
[ "$glob_count" -gt "$GLOB_MAX" ] && viol=$((viol + 1))
[ "$key_count" -gt "$KEY_MAX" ] && viol=$((viol + 1))
[ "$score" -gt "$DIFF_MAX" ] && viol=$((viol + 1))

jarray() {
  awk 'BEGIN { printf "[" } { gsub(/\\/, "\\\\"); gsub(/"/, "\\\""); if (seen++) printf ","; printf "\"%s\"", $0 } END { printf "]" }' "$1"
}

if [ "$json" = 1 ]; then
  printf '{"files":{"total":%s,"max":%s,"over":%s},' "$files_total" "$files_max" "$(jarray "$tmp/over_files")"
  printf '"functions":{"total":%s,"max":%s,"over":%s},' "$fn_total" "$FN_MAX" "$(jarray "$tmp/fn_over")"
  printf '"globs":{"count":%s,"locations":%s},' "$glob_count" "$(jarray "$tmp/globs")"
  printf '"string_keys":{"count":%s,"locations":%s},' "$key_count" "$(jarray "$tmp/keys")"
  printf '"tests":{"total":%s,"max":%s,"over":%s},' "$tests_total" "$TEST_MAX" "$(jarray "$tmp/test_over")"
  printf '"diff":{"base":"%s","added":%s,"removed":%s,"untracked":%s,"score":%s},' "$base" "$added" "$removed" "$untracked" "$score"
  printf '"layers":{"present":%s,"violations":%s},' "$layers_present" "$(jarray "$tmp/layer_viol")"
  printf '"violations":%s}\n' "$viol"
else
  echo "measure: src=$src tests=$tests base=$base"
  echo "[1] files: $files_total total, max $files_max lines, over $FILE_MAX: $over_file_count"
  sed 's/^/    /' "$tmp/over_files"
  echo "[2] functions: $fn_total total, over $FN_MAX: $fn_over_count"
  sed 's/^/    /' "$tmp/fn_over"
  echo "[3] glob imports: $glob_count (limit $GLOB_MAX)"
  sed 's/^/    /' "$tmp/globs"
  echo "[4] string keys: $key_count (limit $KEY_MAX)"
  sed 's/^/    /' "$tmp/keys"
  echo "[5] tests: $tests_total total, over $TEST_MAX: $test_over_count"
  sed 's/^/    /' "$tmp/test_over"
  echo "[6] diff vs $base: added $added, removed $removed, untracked $untracked, score $score (limit $DIFF_MAX)"
  if [ "$layers_present" = 1 ]; then
    echo "[7] layers: violations $layer_count"
  else
    echo "[7] layers: none (no store/model/ops/views files)"
  fi
  sed 's/^/    /' "$tmp/layer_viol"
  echo "violations: $viol"
fi

[ "$viol" -gt 0 ] && exit 1
exit 0
