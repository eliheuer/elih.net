#!/bin/sh
# Copy xix web bundles and example sources into the site, and regenerate the
# code excerpts the post embeds.
# Usage: scripts/sync-xix.sh <example> [more...]
set -e
cd "$(dirname "$0")/.."
xix="${XIX_DIR:-$HOME/GH/repos/xix}"
# Sources and excerpts live outside src/content so the blog collection glob ignores them.
post="src/components/xix"
for ex in "$@"; do
  mkdir -p "public/xix/$ex" "$post"
  cp "$xix/dist/web/$ex/${ex}_web.js" "$xix/dist/web/$ex/${ex}_web_bg.wasm" "public/xix/$ex/"
  cp "$xix/xilem/examples/$ex.rs" "$post/$ex.rs"
  echo "synced $ex ($(du -h "public/xix/$ex/${ex}_web_bg.wasm" | cut -f1))"
done
# Excerpts: <name>-<label>.mdx holds one fenced block between two markers of the source.
python3 - "$post" <<'PY'
import sys, pathlib
post = pathlib.Path(sys.argv[1])
excerpts = {
    "tokens-card": ("tokens.rs", "fn card", "fn app_logic"),
    "tokens-theme": ("tokens.rs", "/// The theme", "struct App"),
}
for name, (src, start, end) in excerpts.items():
    text = (post / src).read_text()
    a, b = text.index(start), text.index(end)
    snippet = text[a:b].rstrip()
    (post / f"{name}.mdx").write_text("```rust\n" + snippet + "\n```\n")
    print(f"excerpt {name}: {snippet.count(chr(10)) + 1} lines")
PY
