# Follow-up opinion: Fable vs Astra

Fable 5.1, 2026-10-08. Based on the rendered captures in `fable/` and `astra/`, `astra/review.md`, `astra/interaction-validation.json`, `parent-review.md`, and a bounded read of `astra/mockup.html`. Opinion only; nothing else was changed.

## Verdict

**Astra as the base, with two of its five changes amended.** Astra is my layout with better defaults, and the screenshots support three of its findings. It is not a different design, so "which do you like best" is really "which defaults". Astra's are closer to what the user wants.

## Astra's corrections

1. **Both sidebars open by default in both views: accept.** My Develop render (`fable/develop-dark-1600x789.png`) shows the collapsed sidebar as twelve identical album icons with no labels. That is an observed defect, not a reviewer's taste. Astra's Develop render keeps readable sources at 246 px and the stage still fits a 6000×4000 photo at ~815×545 px at 1600×789. The cost is at 1280×800 (`astra/develop-light-1280x800.png`): the photo is ~650 px wide, under half the window. Accept, with the responsive rule below.
2. **Four primary tools plus "More tools": accept the idea, reject the split.** My ten-icon rail is uniformly grey and hard to scan in the render; Astra's four read instantly. But Astra hides Remove & heal and Presets, which are everyday actions, and its rail code shows no pressed state when a hidden tool's panel is open: open History from the menu and no rail item is lit (`renderRail`, `astra/mockup.html:1074-1079`). Recommend six visible: Edit, Crop, Heal, Mask, Presets, Info; More holds Red eye, Versions, History, Keywords, and the More button must show pressed + fuse when one of those panels is active.
3. **292 px inspector, 246 px sidebar, one-row footer, 108 px filmstrip: accept with one caveat.** Sliders and values still fit at 292 in the 1280 render. The one-row footer is achieved by `font-size:0` below 1300 px, so Before/After, Clipping and Navigator become unlabelled icons exactly on laptop screens. Keep the row, but keep the Before/After label and move Navigator into the zoom menu.
4. **Muted-text contrast, selection ring, gutters: accept.** Counts, filmstrip position and section labels are visibly more legible in Astra's dark renders; the active-cell ring reads better against the lavender cells.
5. **Palette focus restore: accept.** My version failed the check; Astra's passes.

I also accept the evidence corrections: double-click and the range-key guard pass under the corrected harness; the Temp 2000 claim was not proven; the engine owns processing and nothing in either mockup proves runtime smoothness.

## Remaining UX issues, by user impact

1. **Observed: photo width at 1280×800 with both panels open (~650 px).** Fix: auto-collapse the source sidebar to the rail below ~1366 px of plane width, and add a single "Focus" shortcut (hide both panels, keep filmstrip chip). The filmstrip chip already preserves the return path, so this is safe. Astra built and then disabled a compact-source control (`#compact-source … display:none!important`), which suggests the collapsed state still needs a design.
2. **Observed: filmstrip badges crowd at 66 px thumbnails.** In both 1600×789 renders the pick, star-count and edited glyphs overlap at bottom-left. Show only flag + one star count, drop the edited glyph from the strip.
3. **Untested hypothesis: slider stability in production.** Both mockups own the draft value during a gesture, but no capture can show whether the snapshot path repaints rows mid-gesture. This must be measured on the real app after the runtime fixes, not inferred from the mock.

Minor, observed: every render shows "Illustrative — pixels not readable here" in the histogram because `file://` taints the canvas; production must bind the engine histogram and the clipping triangles to the real payload.

## Defaults recommendation

- **Library:** source sidebar 246 px open; Info panel 292 px open; justified grid; rail with Info active.
- **Develop:** source sidebar open, inspector open, filmstrip 108 px with the source/filter chip; six-tool rail plus More; Detail mode; Fit zoom.
- **Both sidebars:** open by default at ≥1366 px plane width; below that, the source sidebar collapses to the rail with the filmstrip chip as the return path; ⌘\ and ⌘] override and are remembered per view; one Focus shortcut hides both.

## Implementation priority after the runtime fixes

1. Slider component that owns its value for the gesture lifetime (begin / set / end) and measured against real snapshot timing.
2. Single rail + titlebar Library/Develop switch; remove the duplicate strips and the Library footer.
3. Filmstrip bound to the same view slice as the grid, with the context chip.
4. Import review sheet phases and the three-place progress reporting.
5. Responsive sidebar rule and Focus shortcut.
6. Engine histogram with clipping indicators.

## Summary for the user

Take Astra as the base. Keep both sidebars open as you prefer, but let the left one auto-collapse on small windows. Put six tools in the rail, not four, and make the More button show when one of its panels is active. Keep the one-row footer but keep the Before/After label. The real remaining unknown is slider smoothness in the running app, which no mockup can settle.
