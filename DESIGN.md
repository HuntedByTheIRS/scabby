# Scabby design system

The contract for every surface in `web/`. Component code follows this file; when the two
disagree, this file is wrong and gets updated in the same change that moved the component.

## Direction

**Void console.** A developer tool that reads like deep space with a violet signal color. The
ground is nearly black with a purple bias in the hue, surfaces separate in two steps instead of
piling shadows, and purple is spent only where the user is meant to act or look. Data-heavy
surfaces (diffs, SHAs, file trees, counts) are set in mono with tabular numerals so columns line
up on their own.

The model-default we are not building: the cream-ground editorial portfolio look (serif display
face, terracotta accent, generous cream margins). It suits a studio site and is a failure mode on
a code host, which is dense, dark, and read for hours. Also rejected: framework blue, glass
panels, gradient soup, emoji as icons, bouncing easing, uniform card grids, centered-everything
heroes, and shadows on every raised surface.

## Palette

Dark only, for now. Custom user themes are a planned feature, so every color in the app comes
from a token in `web/src/app.css`; no component hard-codes a hex value.

| token | value | use |
| --- | --- | --- |
| `--void` | `#04030A` | page ground |
| `--base` | `#0A0714` | app shell, sticky headers |
| `--surface-1` | `#120B1E` | cards, panels, list rows |
| `--surface-2` | `#1A1029` | raised: hovered rows, popovers |
| `--surface-3` | `#221533` | inputs, code blocks, chips |
| `--line` | `#241634` | hairlines, table rules |
| `--line-strong` | `#3B2459` | active edges, section dividers |
| `--text` | `#EDE9F7` | body copy |
| `--text-muted` | `#9A8FB5` | secondary copy, metadata |
| `--text-faint` | `#6B5F85` | timestamps, placeholders |
| `--purple` | `#8B5CF6` | primary action, selected state, links |
| `--purple-hi` | `#A78BFA` | hover on purple, focus ring |
| `--purple-deep` | `#5B21B6` | pressed, gradient tail, logo mark |
| `--lilac` | `#C7B0FF` | important items: active nav, badges, state labels, key numbers |
| `--glow` | `rgba(139, 92, 246, 0.35)` | the single ambient glow behind the hero |
| `--ok` | `#34D399` | open state, additions, success |
| `--warn` | `#F5A524` | warnings, ahead/behind counts |
| `--danger` | `#F4446A` | destructive actions, deletions, closed state |
| `--add-bg` | `rgba(52, 211, 153, 0.10)` | diff addition rows |
| `--del-bg` | `rgba(244, 68, 106, 0.10)` | diff deletion rows |

Contrast: body copy on any surface is >= 7:1, muted copy >= 4.5:1, and no state is communicated
by color alone (every state also carries a label or an icon).

## Typography

```
--font-sans: Inter, system-ui, -apple-system, "Segoe UI", sans-serif
--font-mono: "JetBrains Mono", "SFMono-Regular", Menlo, Consolas, monospace
```

Scale, in rem, with the role it plays:

| size | use |
| --- | --- |
| 0.75 | metadata, badges, table headers (uppercase, 0.06em tracking) |
| 0.8125 | secondary copy, sidebars |
| 0.875 | body default, list rows, tables |
| 1 | panel titles, form labels |
| 1.25 | page section headings |
| 1.5 | page titles |
| 2 | landing subheads |
| 2.75 | landing display (tight leading, no serif) |

Body text is 14px, not 16px: this is a dense tool. Line height 1.5 for prose, 1.35 for rows.
Mono sets every SHA, path, count, ref name, diff line, and code block, with
`font-variant-numeric: tabular-nums`. No display face beyond weight and size: headings are Inter
at 600, landing display at 700 with -0.02em tracking.

## Space, shape, motion

- Spacing scale: 4, 8, 12, 16, 24, 32, 48, 64. Page gutters 24 (mobile) / 32 (desktop), max
  content width 1200, prose column capped at 72ch.
- Radii: 6 (inputs, buttons), 10 (cards, panels), 14 (modals, hero panels), 999 (pills,
  avatars). One radius per surface, never nested inside itself at the same value.
- Elevation: hairline borders and surface steps do the work. One shadow token
  (`--shadow-pop`: `0 12px 32px rgba(4, 3, 10, 0.7)`) is allowed on popovers and modals only.
- Motion: 120ms for state changes, 180ms for entrances, `cubic-bezier(0.2, 0, 0, 1)`. No bounce,
  no parallax, no scroll-jacking, nothing over 240ms. Every animation is wrapped in
  `@media (prefers-reduced-motion: reduce)` and collapses to opacity only.
- Focus: 2px ring in `--purple-hi` at 60% opacity, 2px offset, on `:focus-visible` only.

## Layout

Three shells:

1. **Public** (landing, login, register): no sidebar, centered column, ambient `--glow` radial
   behind the top third, footer with docs/source/license links.
2. **App** (dashboard, settings, new repo): sticky top bar (48px) + 260px sidebar that collapses
   to icons under 1024px and to a drawer under 768px.
3. **Repo** (code, commits, issues, settings): top bar + repo header band (name, visibility pill,
   star/clone actions, tab row: code, commits, issues, settings). The tab row scrolls
   horizontally on mobile; the sticky sub-header carries breadcrumbs and the ref picker.

Breakpoints: 375 (narrow), 768 (tablet), 1024 (sidebar collapse), 1280 (full: sidebar + gutter),
1440 (max content). Nothing in the app relies on hover for a primary action; every hover state has
a focus equivalent.

## Component inventory

Every primitive below ships with default, hover, focus-visible, active, disabled, loading, empty,
and error states where the state is meaningful. States are in the component, not the page.

| component | notes |
| --- | --- |
| `Button` | variants `primary` (purple fill), `secondary` (surface-2 + hairline), `ghost`, `danger`. Sizes `sm`/`md`. Loading swaps the label for a spinner and keeps the width stable. |
| `IconButton` | square, tooltip on hover and focus. |
| `Input`, `Textarea`, `Select` | surface-3 ground, hairline, purple focus ring, inline error text under the field with `aria-describedby`. |
| `Card` | surface-1 panel, 10px radius, optional header row with a title and trailing actions. |
| `Badge` | visibility, issue state, ref name. `lilac` for important state, `ok`/`danger` for open/closed, neutral for metadata. |
| `Avatar` | deterministic two-color gradient keyed off the username hash, initials in mono, sizes 24/32/48/96. |
| `Tabs` | repo header and settings; underline on the active tab in `--purple`, `aria-current` set. |
| `Table` / `ListRow` | hairline rules, no zebra striping, hover raises to surface-2, full-row click target with a real focusable link inside. |
| `FileTree` | mono paths, chevron disclosure, directory rows bold-ish (500), file rows muted. |
| `DiffView` | unified diff, `--add-bg`/`--del-bg` per line, line-number gutters in `--text-faint`, hunk headers in lilac. |
| `EmptyState` | one line of what is missing, one primary action, no illustration. |
| `Spinner` / `Skeleton` | skeleton rows for lists, spinner for buttons and inline actions. |
| `Toast` | bottom-right, surface-2, hairline, auto-dismiss 4s, never for errors that need a decision. |
| `Modal` | only for irreversible actions (delete repo, delete token), `--shadow-pop`, escape closes, focus trapped. |
| `CopyField` | clone URL: mono, truncated in the middle, copy button that confirms in place. |

## Page and state matrix

| route | 375 | 768 | 1280 | empty | loading | error |
| --- | --- | --- | --- | --- | --- | --- |
| `/` landing | stacked, mono logo | 2-col feature grid | 3-col + code sample | n/a | n/a | n/a |
| `/login`, `/register` | full-bleed form | centered card 400px | same | n/a | button spinner | inline field errors, form-level alert for bad credentials |
| `/dashboard` | list of repos | + activity feed | sidebar + repo grid + activity | "No repositories yet" + create action | skeleton rows | retry banner |
| `/new` | stacked form | 2-col form | same, preview panel | name availability hint | button spinner | field errors from API `errors[]` |
| `/settings` | tabs as pills | sidebar tabs | sidebar tabs + content | keys/tokens empty state with the exact command to add one | skeleton | retry banner |
| `/[owner]` | avatar header + repos | + bio + meta | + pinned repos, contribution counts | "No public repositories" | skeleton | 404 panel with a search link |
| `/[owner]/[repo]` | header collapses, tabs scroll | file list + README | + sidebar (about, clone, languages) | "Empty repository" with the push instructions | skeleton | 404 or 403 panel |
| `/[owner]/[repo]/tree/[ref]/[...path]` | single column | tree + last-commit column | + README pane | empty directory row | skeleton | "path not found at this ref" |
| `/[owner]/[repo]/commits` | 2-line rows | + author | + date, graph dot | "No commits" | skeleton | retry |
| `/[owner]/[repo]/commit/[sha]` | diff scrolls horizontally | 2-pane summary | summary + full diff | n/a | skeleton | unknown revision panel |
| `/[owner]/[repo]/issues` | list, filter chips | + labels column | + counts strip | "No open issues" + new issue action | skeleton | retry |
| `/[owner]/[repo]/issues/[number]` | stacked | + sidebar | + timeline | "No comments yet" | skeleton | 404 |
| `/[owner]/[repo]/settings` | stacked sections | + danger zone | same | n/a | button spinner | field errors, confirm modal on delete |

Locale: no CJK-specific work in this pass. If Korean or Chinese copy lands, body text holds a
14px floor and mono labels are not used for CJK text.

## Performance and accessibility budget

- LCP: the landing headline, on a 4x CPU throttle and Slow 4G, under 2.0s. Budget is the first
  paint of the SPA shell: JS under 180KB gzipped for the landing route.
- INP under 200ms on the repo list and issue list interactions; no work over 50ms on the main
  thread, so long lists render in pages of 30 rather than all at once.
- CLS under 0.1: avatar, badge, and diff-gutter dimensions are fixed, skeletons match the real
  row height.
- Keyboard: every route reachable by tab, skip-to-content link, visible focus everywhere, modals
  trap focus and restore it, `aria-current="page"` on nav, `aria-live="polite"` for toasts and
  inline validation.
- Every image and icon has an accessible name or is `aria-hidden`; state is never color-only.

## Anti-slop checklist

Rejected on sight in review: a purple gradient on every surface, glassmorphism, drop shadows on
cards, emoji used as UI icons, decorative blobs, placeholder lorem copy, centered text over three
lines, a dashboard that is a grid of identical stat cards, a landing page that says "blazingly
fast" without a number, and any component whose only state is the default one.

## Known gaps in the built interface

Measured against the running bundle, not guessed. Each of these is a gap between this document and
what ships today; none of them is fixed by claiming it works.

- **The type is specified, not shipped.** Inter and JetBrains Mono appear only as family names in
  `web/src/app.css`; there is no `@font-face`, no self-hosted file and no font package. On a machine
  without both fonts installed, the interface renders in `system-ui` and the generic monospace.
- **No ref picker.** The client can list branches and tags and the server serves `/branches` and
  `/tags`, but no page calls them: repository routes take an explicit `?ref=` and otherwise fall back
  to the default branch. The repository sub-header described above therefore has no picker yet.
- **No per-path last commit.** The tree view wants a last-commit column; the API exposes only the
  repository's head commit, so the column renders an em dash with a tooltip rather than invented data.
- **No per-line diff.** The commit page shows the file list with real additions and deletions and
  says outright that this API version returns no patches. See the note next to `files[]` in
  `docs/api.md`.
