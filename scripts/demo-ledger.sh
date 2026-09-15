#!/usr/bin/env bash
# Build a small English demo ledger to try `gy serve` on (d-72af): a fictional
# project, "Orchard", a reading-list web app. It uses gy's own CLI, needs no
# network, and writes the same content on every run. The ids differ, because
# gy mints them.
set -euo pipefail

dir=$(mktemp -d "${TMPDIR:-/tmp}/gy-demo.XXXXXX")
repo="$dir/repo"
data="$dir/data"
mkdir -p "$repo" "$data" "$dir/bodies"
printf 'output = "pub"\n\n[scopes.orchard]\n[scopes.billing]\n' > "$repo/gy.toml"

# gy from PATH, else the release build, else cargo run.
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
root=$(dirname "$here")
if command -v gy >/dev/null 2>&1; then
  GY=(gy)
elif [ -x "$root/target/release/gy" ]; then
  GY=("$root/target/release/gy")
else
  GY=(cargo run -q -p gy --manifest-path "$root/Cargo.toml" --)
fi

export XDG_DATA_HOME="$data" GY_ACTOR=demo
w() { "${GY[@]}" -C "$repo" "$@"; }
# The id of a write, read from its JSON.
id() { w "$@" --json | sed -n 's/.*"id":"\([^"]*\)".*/\1/p'; }
o() { w --scope orchard "$@"; }
b() { w --scope billing "$@"; }
oid() { id --scope orchard "$@"; }
bid() { id --scope billing "$@"; }
# Give a node an English body, as one write.
body() {
  printf '%s\n' "$2" > "$dir/bodies/body.txt"
  w edit "$1" --reason "record the why" --body-file "$dir/bodies/body.txt" >/dev/null
}

# Acceptance criteria: orchard keeps the list, billing the money.
c1=$(oid criterion add "A shared list opens in under 300 ms with 1,000 items")
c2=$(oid criterion add "An invite link expires after seven days")
c3=$(oid criterion add "An import keeps the folder each bookmark came from")
c4=$(oid criterion add "Deleting a list removes its shared links")
c5=$(bid criterion add "An invoice shows the item count it charged for")
c6=$(bid criterion add "A retry never charges the same list twice")
oid criterion satisfy "$c1" --evidence "p95 241 ms over 50 runs on the CI machine" >/dev/null
bid criterion satisfy "$c6" --evidence "the retry ledger shows one charge for each list id" >/dev/null

# Questions: two for the master stay open, one closes by a decision, one for lead.
q1=$(oid question add "Should a shared list be editable by the recipient?" --decider master \
  --options "Read only" --options "Edit by anyone with the link" \
  --options "Edit by a named collaborator")
q2=$(oid question add "Should an expired invite show a re-request page?" --decider master \
  --options "Show a re-request page" --options "Show a plain 404")
q3=$(oid question add "Should an import keep the original bookmark order?" --decider master \
  --options "Keep the file order" --options "Sort by title")
q4=$(bid question add "Should the invoice email attach the PDF?" --decider lead \
  --options "Attach the PDF" --options "Link to it instead")

# Decisions: one closes q1, two narrow an older decision, several spawn needs.
d1=$(oid decide "A share link is a URL with a signed token" \
  --scope-note "A shared list needs no account: the URL carries a signed token with the read scope and an expiry.")
d2=$(bid decide "Billing counts a list when it is first shared" \
  --scope-note "A list is billable on its first share; keeping many private lists stays free.")
d3=$(oid decide "The share token lives in the URL fragment" \
  --scope-note "The token goes in the fragment so it never reaches the server access log." \
  --relate narrows "$d1" --mark "the read scope and an expiry")
d4=$(oid decide "An import runs on the server" \
  --scope-note "The server parses the bookmark file so one code path serves every browser.")
d5=$(oid decide "The folder comes from the file's folder attribute" \
  --scope-note "Each imported link keeps the folder recorded in its bookmark file." \
  --relate narrows "$d4" --mark "the bookmark file")
d6=$(oid decide "Read-only sharing for the beta" --closes "$q1" \
  --scope-note "Editing by link raises a permissions model we do not have yet, so the beta is read only.")
d7=$(bid decide "An issued invoice never changes" \
  --scope-note "Once an invoice is issued it is immutable; a correction is a new invoice.")
d8=$(oid decide "Deleting a list deletes its tokens" \
  --scope-note "Deleting a list also deletes every share token that points at it.")

# Needs: six, each against criteria; one closed by a fact.
n1=$(oid need add "Share a list by link" --targets "$c1" --targets "$c2" --spawned-by "$d1")
n2=$(oid need add "Import from a browser's bookmarks" --targets "$c3" --spawned-by "$d1")
n3=$(oid need add "Delete a list cleanly" --targets "$c4")
n4=$(bid need add "Show the item count on an invoice" --targets "$c5" --spawned-by "$d2")
n5=$(bid need add "Retry a failed charge safely" --targets "$c6" --spawned-by "$d2")
n6=$(bid need add "Charge for lists over 50 items" --targets "$c5" --targets "$c6")
b need close "$n6" --by fact --evidence "the 50-item cap shipped and the invoice shows the count" >/dev/null

# Requirements: one waits for the master, one is approved.
r1=$(oid req add "Ship share-by-link for the beta" --need "$n1" \
  --relies-on "$d1" --relies-on "$d3" --targets "$c1" --targets "$c2" \
  --ref "https://example.com/orchard/issues/12")
r2=$(bid req add "Bill for lists over 50 items" --need "$n4" --relies-on "$d2" \
  --targets "$c5" --ref "https://example.com/orchard/issues/31")
b req approve "$r2" --design "The count on the invoice is the number of shared lists at issue time." \
  --heard-by "master" --evidence "approved in the billing review" >/dev/null

# The why and the story, so the node pages read well.
body "$c1" "Measure the time to first render of a 1,000-item list on the CI machine."
body "$c2" "Issue a token today and check it fails after seven days and works before."
body "$c3" "Import a bookmark file and compare each folder with the source tree."
body "$c4" "Share a list, delete it, and open each former link."
body "$c5" "Compare the invoice's item count with the number of shared lists at issue."
body "$c6" "Replay the retry ledger and count the charges for each list id."
body "$d1" "The token is signed with the list id and an expiry, so no session is needed."
body "$d2" "A list is billable on its first share; the count is the number of shared lists."
body "$d3" "Fragments stay in the browser, so the token never lands in the server log."
body "$d4" "Parsing on the server keeps one code path for every browser's bookmark format."
body "$d5" "The folder attribute is the only place the original tree survives; keep it verbatim."
body "$d6" "Editing by link needs a permissions model we do not have yet, so the beta is read only."
body "$d7" "A correction is issued as a new invoice that references the one it replaces."
body "$d8" "A token that outlives its list is a leak, so deletion sweeps them in one transaction."
body "$n1" "A reader can open a list without an account when the owner sends them a link."
body "$n2" "People moving from another browser want their folders to arrive intact."
body "$n3" "Deleting a list must not leave a working share link behind."
body "$n4" "The invoice is the only place a customer sees what they were charged for."
body "$n5" "A timed-out charge must be safe to retry without a second charge."
body "$n6" "Lists above the free limit are billed per shared list."

echo "ledger: $repo"
echo "try: gy -C $repo serve"