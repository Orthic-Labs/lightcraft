# Astra adversarial review

## Findings

1. **High — Develop source context was too weak.** Fable’s collapsed left rail became an icon train, so return path depended on memory. Astra opens left source sidebar by default in Develop, keeps exact source + filter in visible filmstrip context chip, & retains sidebar toggle for compact mode.

2. **High — Tool rail carried too much equal visual weight.** Ten icon-only actions made primary editing ambiguous. Astra keeps Edit, Crop, Masking & Info visible, moves advanced actions into named keyboard-accessible `More tools` menu, & preserves every panel.

3. **Medium — Photo workspace lost width at smaller frames.** Fable used 320 px inspector + 268 px sidebar. Astra measures 292 px inspector + 246 px sidebar; filmstrip context remains visible.

4. **Medium — Library density weakened culling rhythm.** Astra increases cell gutters, strengthens selected-photo ring, adds calmer title/filter insets, & keeps metadata inspector readable without KPI cards.

5. **Medium — Palette Escape dropped opener focus.** Astra stores opener before palette opens & restores focus on Escape, backdrop close, or command run.

## Concrete revision

`mockup.html` is copied from Fable baseline, then bounded CSS, rail markup, source-return markup, rail disclosure, & palette focus edits were applied. RightKit shell tokens, real LightCraft image assets, semantic controls, filmstrip source/filter row, slider gesture guard, modal trap, reduced-motion rules, & simulation honesty remain intact.

## Verification

- `node review/design-refresh/render-design.mjs review/design-refresh/astra`: **12 captures**, **0 errors**, no horizontal overflow at 1600×1000, 1280×800, or 1600×789, both themes/views; Develop inspector measured **292 px**, stage footer **one row** at 1280×800.
- `node review/design-refresh/astra/check-interactions.mjs`: **12/12 checks pass** — double-click Library→Develop, filmstrip step, slider ArrowRight guard, import inert/focus/restore, palette focus restore, More tools disclosure, left source + filmstrip context, optional sidebar toggle, measured 1280×800 + 1600×789 geometry.
- Rust processing, native import/export, Auto quality, & production performance remain outside mockup evidence.
- Geometry receipt at 1280×800: inspector `x=938,w=292,h=748`; stage footer `x=246,y=648,w=692,h=44`; filmstrip `h=108`; body `scrollWidth=clientWidth=1280`.
- Geometry receipt at 1600×789: inspector `x=1258,w=292,h=737`; stage footer `x=246,y=637,w=1012,h=44`; filmstrip `h=108`; body `scrollWidth=clientWidth=1600`.
- Captures inspected: `library-dark-1600x1000.png`, `develop-dark-1600x1000.png`, `develop-light-1280x800.png`, `develop-dark-1600x789.png`, `library-dark-1600x789.png`.
