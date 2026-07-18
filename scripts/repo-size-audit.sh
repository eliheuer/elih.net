#!/bin/sh

set -eu

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

budget_mb=${REPO_BUNDLE_BUDGET_MB:-100}
audit_dir=$(mktemp -d "${TMPDIR:-/tmp}/elih-net-size.XXXXXX")
bundle_path="$audit_dir/main.bundle"

cleanup() {
    rm -f "$bundle_path"
    rmdir "$audit_dir" 2>/dev/null || true
}
trap cleanup EXIT HUP INT TERM

git bundle create "$bundle_path" main >/dev/null

bundle_bytes=$(wc -c < "$bundle_path" | tr -d ' ')
current_bytes=$(git ls-tree -r -l HEAD | awk '$2 == "blob" { total += $4 } END { printf "%.0f", total }')
budget_bytes=$((budget_mb * 1024 * 1024))

to_mib() {
    awk -v bytes="$1" 'BEGIN { printf "%.1f MiB", bytes / 1048576 }'
}

printf 'Compressed main history: %s / %s MiB budget\n' "$(to_mib "$bundle_bytes")" "$budget_mb"
printf 'Current committed tree:  %s\n' "$(to_mib "$current_bytes")"

printf '\nLargest files in the current commit:\n'
git ls-tree -r -l HEAD |
    awk '$2 == "blob" { printf "%12d  %s\n", $4, $5 }' |
    sort -nr |
    head -15

printf '\nBinary paths with the most reachable blob data on main:\n'
git rev-list --objects main |
    git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' |
    awk '$1 == "blob" {
        path = substr($0, index($0, $4));
        if (path !~ /\.(png|jpg|jpeg|gif|webp|mp4|woff|woff2|wasm)$/) {
            next;
        }
        revisions[path] += 1;
        bytes[path] += $3;
    }
    END {
        for (path in bytes) {
            printf "%12.0f  %4d revisions  %s\n", bytes[path], revisions[path], path;
        }
    }' |
    sort -nr |
    head -15

if [ "$bundle_bytes" -gt "$budget_bytes" ]; then
    printf '\nERROR: compressed main history exceeds the %s MiB budget.\n' "$budget_mb" >&2
    exit 1
fi
