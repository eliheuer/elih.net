# Domain Migration: eliheuer.com → elih.net

## Overview

Moving the site to the new `elih.net` domain, into a fresh repo at
`~/GH/repos/elih-dot-net` (GitHub: `eliheuer/elih.net`). The site is an
Astro project hosted on GitHub Pages. DNS is managed through the domain
registrar. The GitHub **username stays `eliheuer`**, so `*.github.io`,
repo URLs, and social handles do not change — only the website domain.

The previous repo (`eliheuer/eliheuer.com`) keeps the full commit history;
this one starts fresh.

---

## Code Changes (done)

Files that referenced `eliheuer.com` as the **site domain**, now `elih.net`:

- **`public/CNAME`** — GitHub Pages custom domain file → `elih.net`. Moved from
  the repo root into `public/` so the build emits it to `dist/CNAME`; with
  Actions-based deploys this auto-configures the Pages custom domain and keeps it
  from being dropped on redeploy.
- **`astro.config.ts:24`** — `site` → `https://elih.net`.
- **`src/consts.ts:7`** — `href` → `https://elih.net`.
- **`package.json:2`** — `name` → `elih.net` (cosmetic).
- **`public/runebender-web/index.html`** — 4 `og:`/`twitter:` URLs → `elih.net/runebender-web/`.

Deliberately **not** changed (these are the handle/username, not the domain):
`github.com/eliheuer/*`, `@eliheuer` social links, `eliheuer.github.io`.

---

## DNS Setup (registrar → Advanced DNS for `elih.net`)

Remove any default parking/redirect records, then add:

### A Records (apex domain → GitHub Pages)

| Type | Host | Value           |
| ---- | ---- | --------------- |
| A    | @    | 185.199.108.153 |
| A    | @    | 185.199.109.153 |
| A    | @    | 185.199.110.153 |
| A    | @    | 185.199.111.153 |

(Optionally also add the four AAAA records for IPv6:
`2606:50c0:8000::153`, `…8001::153`, `…8002::153`, `…8003::153`.)

### CNAME Record (www subdomain)

| Type  | Host | Value              |
| ----- | ---- | ------------------ |
| CNAME | www  | eliheuer.github.io |

---

## GitHub Pages Settings (new repo)

Go to `eliheuer/elih.net` → **Settings → Pages** and:

1. Confirm **Source** = "GitHub Actions" (the deploy workflow handles the build).
2. The **Custom domain** should auto-populate to `elih.net` from the `CNAME`
   file on first deploy; if not, set it manually.
3. Check **Enforce HTTPS** once DNS has propagated and the cert is issued.

---

## Old domain (`eliheuer.com`)

GitHub Pages allows only one custom domain per repo, so once `elih.net` is
live the old `eliheuer.com` will stop resolving unless handled. Options:

- **Retire it** — let it lapse / point nowhere.
- **Redirect** — keep the old repo deployed with a redirect to `elih.net`,
  or set a registrar-level URL redirect `eliheuer.com → elih.net`.

(Decide later — not required for `elih.net` to go live.)

---

## Checklist

- [x] Copy source into `~/GH/repos/elih-dot-net` (fresh, no history)
- [x] Update `CNAME` to `elih.net`
- [x] Update `astro.config.ts` site URL to `https://elih.net`
- [x] Update `src/consts.ts` href to `https://elih.net`
- [x] Update `package.json` name to `elih.net`
- [x] Update `public/runebender-web/index.html` OG/Twitter URLs
- [x] `pnpm install` + `pnpm build` verify
- [x] `git init`, commit
- [x] Create `eliheuer/elih.net` on GitHub and push
- [ ] Configure DNS A records at registrar (4 GitHub Pages IPs)
- [ ] Configure DNS CNAME record (`www` → `eliheuer.github.io`)
- [ ] Confirm GitHub Pages custom domain = `elih.net`
- [ ] Enable "Enforce HTTPS" once DNS propagates
- [ ] Verify site loads at `https://elih.net`
- [ ] Decide fate of old `eliheuer.com` (retire vs redirect)
