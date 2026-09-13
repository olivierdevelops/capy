# Release-readiness status (v0.1.0)

A snapshot of where the repo stands against [RELEASE_PLAN.md](RELEASE_PLAN.md).
**Repo: https://github.com/olivierdevelops/capy**
**Docs site: https://olivierdevelops.github.io/capy/** (auto-deployed on push to main)

---

## Phase-by-phase status

| Phase | Item | Status |
|-------|------|--------|
| **0 — Hygiene** | LICENSE, CHANGELOG, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, .gitignore, README, issue/PR templates, CODEOWNERS | ✅ |
| **1 — Tests + CI** | Unit tests + 116 golden tests + wasm check + matrix CI (Linux/macOS/Windows) | ✅ |
| **2 — CLI** | `capy run/check/init/version/help` + caret-pointed errors | ✅ |
| **3 — Docs** | 16 doc pages + 4 tutorials + **50 sample demos** with verified goldens | ✅ |
| **4 — Tooling** | JSON schema + VS Code extension | ✅ (tree-sitter deferred) |
| **5 — Distribution** | GoReleaser + Dockerfile + install script + release workflow | ✅ (Homebrew tap stub) |
| **6 — AI ecosystem** | Claude skill + 5 slash commands + Cursor/Continue/Aider snippets + LLM brief | ✅ |
| **7 — Launch drafts** | Blog post + HN + Reddit + Twitter drafts | ✅ |
| **8 — Docs site** | MkDocs Material, deploys to GitHub Pages via Actions | ✅ |

---

## Final pre-tag punch list

| # | Action | Done? |
|---|--------|-------|
| 1 | Replace `capyhq` placeholder org with `olivierdevelops` (~20 files) | ✅ |
| 2 | Cargo workspace under `rust/` (`capy-core`, `capy-cli`, `capy-mcp`) | ✅ |
| 3 | Update `SECURITY.md` reporting channel | ✅ (uses GitHub Security Advisories) |
| 4 | Decide install URL + update README/install.sh | ✅ (uses raw.githubusercontent.com) |
| 5 | MkDocs config + GitHub Pages workflow | ✅ |
| 6 | Create `olivierdevelops/homebrew-tap` repo, uncomment `brews:` block | ⏳ optional, can defer |
| 7 | Push to GitHub, enable Discussions | ⏳ on you |
| 8 | Test a release build locally: `cargo build --release --manifest-path rust/Cargo.toml` | ⏳ on you |
| 9 | `git tag v0.1.0 && git push --tags` → release workflow auto-publishes | ⏳ on you |
| 10 | Publish blog post → HN → Reddit → social | ⏳ from drafts in `docs/launch/` |

---

## Quantitative snapshot

```
50  demos (samples/), each with verified golden output
52  golden tests passing
20  user-facing docs pages (incl. tutorials)
 4  progressive tutorials
20  AI / agent integration files (skills, commands, editors, system prompt)
 9  GitHub workflow + CODEOWNERS + issue templates
 0  cargo clippy / cargo build / cargo test failures
 0  remaining placeholders in real project URLs
```

---

## GitHub Pages site (auto-deployed)

- **Source**: `mkdocs.yml` at repo root + everything in `docs/`
- **Theme**: MkDocs Material (light + dark, instant nav, code copy, search)
- **Workflow**: `.github/workflows/docs.yml` — triggers on push to main
- **URL**: https://olivierdevelops.github.io/capy/

The nav structure follows the docs layout:

- **Home** — landing page
- **Getting started** — five-minute tour
- **Guide** — library authoring, language ref, inner DSL, types, templates, blocks, transpiler patterns, CLI
- **Tutorials** — four progressive walkthroughs
- **Reference** — cookbook, FAQ, architecture, roadmap, migration guide
- **For AI agents** — CAPY_FOR_LLMS.md (single-page brief)

Internal-only notes (`docs/launch/*`, `docs/legacy/*`) are excluded from the site via `not_in_nav:`.

---

## When you're ready

The actual day-of-release sequence (1–2 hours):

```sh
# 1. push to GitHub
git add -A && git commit -m "v0.1.0 release prep"
git push origin main

# 2. wait for CI to go green (ci.yml + docs.yml)
#    - CI: tests, lint, build
#    - docs.yml: deploys to https://olivierdevelops.github.io/capy/

# 3. enable GitHub Discussions in repo Settings

# 4. (optional) test a release build locally
cargo build --release --manifest-path rust/Cargo.toml
# confirms dist/ has all 5 platform binaries

# 5. cut the release
git tag v0.1.0
git push origin v0.1.0
# → release.yml cross-compiles with cargo, publishes binaries to Releases

# 6. test install script against the new release
curl -fsSL https://raw.githubusercontent.com/olivierdevelops/capy/main/scripts/install.sh | sh

# 7. publish launch material
#    - blog post  → from docs/launch/blog-post.md
#    - HN         → from docs/launch/hn.md (Tue/Wed AM PT)
#    - Reddit     → from docs/launch/reddit.md (after ~24h)
#    - social     → from docs/launch/twitter.md
```

---

## Open optional items (low priority)

- **Homebrew tap.** Create `olivierdevelops/homebrew-tap` repo and add a formula pointing at the release archives. Until then, install via `cargo install` or the release-binary script.
- **Logo + demo GIF.** Drop in `assets/`. Needs design / terminal recording.
- **Publish VS Code extension to Marketplace.** Sources are ready in `editors/vscode/capy/`.
- **`awesome-capy` repo.** Curated list of community libraries.
- **Tree-sitter grammar.** Tracked in [roadmap](docs/roadmap.md).
