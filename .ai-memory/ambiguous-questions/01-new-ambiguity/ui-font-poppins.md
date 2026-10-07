# Ambiguity: Poppins body font (spec-04 / RiseUp vs shipped Ubuntu-only)

Status: proceeding by best judgment (plan D4) — Poppins deferred.

## Question

`04-typography.md` and RiseUp `spec/05-brand-system.md` §3 fix Poppins as the body
font (Ubuntu headings only, "never Ubuntu below 18px"). AGM ships Ubuntu for
everything + Ubuntu Mono for code + Effra for RTL (`src/App.css`), with no
Poppins webfont or bundled asset.

## Decision taken

Keep Ubuntu body for this overhaul; do not add a Poppins `<link>`/bundle.
Rationale: new remote-font dependency affects offline/WebView2 startup, RTL
fallback stacks, and every acceptance screenshot; it deserves its own
font-delivery decision (bundle vs CDN, weights, subsets).

## If overturned

Bundle Poppins 400/500/600 (latin subset, `display=swap`), re-point body stacks,
keep Ubuntu Mono code + Effra RTL intact, and re-run T4 plus a cold-start
offline check.
