#!/usr/bin/env bash
# Run a check with Bun, while making Node's command names unavailable.
set -euo pipefail

if [[ $# -eq 0 ]]; then
  printf 'usage: %s command [args...]\n' "$0" >&2
  exit 2
fi

node_free_bin=$(mktemp -d "${TMPDIR:-/tmp}/gy-without-node.XXXXXX")
trap 'rm -rf "$node_free_bin"' EXIT
# Use Bun for the filesystem pass, without launching a process per symlink.
bun -e '
  import { accessSync, constants, readdirSync, statSync, symlinkSync } from "node:fs";
  import { join, resolve } from "node:path";
  const excluded = new Set(["node", "nodejs", "npm", "npx", "corepack"]);
  const held = new Set();
  for (const directory of process.env.PATH.split(":")) {
    if (!directory) continue;
    let names;
    try { names = readdirSync(directory); }
    catch (error) {
      if (["ENOENT", "ENOTDIR", "EACCES"].includes(error.code)) continue;
      throw error;
    }
    for (const name of names) {
      if (excluded.has(name) || held.has(name)) continue;
      const source = resolve(directory, name);
      try {
        if (!statSync(source).isFile()) continue;
        accessSync(source, constants.X_OK);
      } catch (error) {
        if (["ENOENT", "EACCES", "ELOOP"].includes(error.code)) continue;
        throw error;
      }
      symlinkSync(source, join(process.argv[1], name));
      held.add(name);
    }
  }
' "$node_free_bin"

export PATH="$node_free_bin"
hash -r
for runtime in node nodejs; do
  if command -v "$runtime"; then
    printf 'Node-free check refused: %s is still available\n' "$runtime" >&2
    exit 1
  fi
  printf 'command -v %s: unavailable\n' "$runtime"
done
printf 'Bun version: %s\nBun revision: %s\n' "$(bun --version)" "$(bun --revision)"
"$@"
