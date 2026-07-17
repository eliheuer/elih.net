# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

Package manager is **pnpm**. Dev server runs at `http://localhost:1234` (configured in `astro.config.ts`).

| Command | Purpose |
| --- | --- |
| `pnpm dev` (or `pnpm start`) | Start Astro dev server |
| `pnpm build` | Runs `astro check && astro build` — type-checks before bundling |
| `pnpm preview` | Preview the production build |
| `pnpm prettier` | Format `.ts/.tsx/.css/.astro` (config inline in `package.json`) |
| `pnpm astro ...` | Astro CLI passthrough |

There is no test suite. `astro check` is the closest thing to a check command and is part of `pnpm build`.

A `postinstall` step runs `patch-package`, applying `patches/rehype-pretty-code+0.14.1.patch`. Do not skip it — markdown rendering depends on it.

## Deployment

GitHub Pages via `.github/workflows/deploy.yml`: every push to `main` triggers `withastro/action@v4`, which builds with pnpm and deploys with `actions/deploy-pages@v4`. The `CNAME` file pins the custom domain (`elih.net`).

## Architecture

Astro 5 static site (originally forked from the `astro-erudite` template — see `README.md`) with React 19 islands, Tailwind v4 (via `@tailwindcss/vite`), MDX content, KaTeX math, and Expressive Code + `rehype-pretty-code` for code blocks. Path alias `@/*` → `src/*`.

### Content collections

All collections are declared in `src/content.config.ts` using `glob` loaders against `src/content/<name>/`. Five collections, each with its own Zod schema:

- `blog` — `src/content/blog/<slug>/index.mdx` with co-located assets (images, PDFs). Schema: `title`, `description`, `date`, optional `image`, `tags`, `authors`, `draft`.
- `authors` — referenced by id from blog `authors` arrays; resolved by `parseAuthors()`.
- `projects` — used by `projects.astro`. Requires `image` and `link`.
- `fonts` and `software` — same shape as `blog` (no `authors`); each entry is a folder with `index.md` + assets.

Each content type has its own dynamic page route: `src/pages/blog/[...id].astro`, `src/pages/fonts/[...id].astro`, `src/pages/software/[...id].astro`. They use `getStaticPaths` over the collection and call `render(post)`.

`src/lib/data-utils.ts` is the single place that queries collections — `getAllPosts`, `getRecentPosts`, `getAdjacentPosts`, `getPostsByTag`, `getPostsByAuthor`, `groupPostsByYear`, `parseAuthors`. It already filters out drafts. Note the commented-out single-post filter and TODO at the top — the site temporarily showed only one post; restore the filter only deliberately.

`PostHead.astro` accepts entries from `blog | fonts | software` and constructs the OG image URL as `/og/<first-path-segment-of-post.id>.png` from `public/og/`. New posts that need a social card must drop a PNG there with a matching name.

### Site config

`src/consts.ts` holds `SITE`, `NAV_LINKS`, `SOCIAL_LINKS`, and `ICON_MAP` (Lucide icon names used by `SocialIcons.astro`). Types in `src/types.ts`. Update these rather than hardcoding nav/social/site metadata in components.

### Layout & font system

`src/layouts/Layout.astro` is the only layout. It sets either `use-bezy-font` or `use-geist-font` on `<html>` (default: `bezy`) and re-applies the choice from `localStorage` via an inline script before paint to avoid FOUT. `FontToggle.astro` writes to that same key. Custom CSS variants `use-bezy-font` / `use-geist-font` are declared in `src/styles/global.css` so Tailwind utilities can scope per-font.

Font files live in `public/fonts/` and are registered with `@font-face` in `src/styles/global.css`. Adding a new font file means: drop the woff2 in `public/fonts/`, add an `@font-face` block, and (if it should be user-selectable) extend the toggle and the `use-*-font` variant set.

Recent design work (see `git log`) standardized on a **36px baseline grid** with full-width layouts and uniform type sizing. Inline styles in templates and class lists in components reflect this — preserve the rhythm when changing layout-affecting CSS.

### UI components

`src/components/ui/` contains shadcn/ui-style React + Astro primitives (`button`, `badge`, `dropdown-menu`, etc.). Higher-level Astro components (`BlogCard`, `ProjectCard`, `Header`, `Footer`, `TableOfContents`, …) live one level up in `src/components/`. The `cn()` helper in `src/lib/utils.ts` is the standard `clsx + tailwind-merge` combiner.

### Markdown pipeline

Configured in `astro.config.ts`:
- Remark: `remark-math`, `remark-emoji`, `remark-sectionize`
- Rehype: `rehype-document` (injects KaTeX CSS CDN), `rehype-external-links` (opens new tab + `nofollow`), `rehype-heading-ids`, `rehype-katex`, `rehype-pretty-code`
- `astro-expressive-code` is the primary code-block renderer (themes `github-light` / `github-dark`, gated by `.light` / `.dark` class via `themeCssSelector`). Built-in syntax highlight is disabled (`syntaxHighlight: false`) to let Expressive Code own it.

## Writing style (all prose: post body, captions, alt text, titles)

The model is Tao Lin's directness applied to technical writing:
https://www.taolin.us/writing. Say the fact. Stop.

- Short declarative sentences. One idea per sentence. Subject first.
- Plain words: "use" not "leverage", "shows" not "demonstrates".
- State numbers and concrete nouns. Delete intensifiers (very, really,
  deeply) and adjectives that do not carry information.
- No cuteness. No aphorisms, paradox hooks, or applause lines. No
  "here's the trick", "worth slowing down for", "the evidence fits in
  three numbers", "writes itself".
- No anthropomorphizing: points do not "know", the eye does not "ask",
  grids do not "insist".
- No conversational filler: "of course", "honestly", "frankly", "a fair
  objection", "agreed, and that is the point".
- Never em dashes (rewrite with commas, parentheses, colons, or periods).
- Metaphors: at most one per section, only if it carries information a
  plain sentence cannot.
- Do not restate. If a fact appeared earlier, refer to it; do not
  re-explain it.
- Titles are short noun phrases.
- A boring sentence that is clear beats a memorable sentence that is not.
- It is fine to sound flat. Flat is the voice.
- Names: the font is "Virtua Grotesk" (or "the sources"); the model is
  "Virtua-12M-v0.N" (public) with internal run ids (v0.8) only in
  experiment narration. Never bare "Virtua" where font and model could
  be confused.
