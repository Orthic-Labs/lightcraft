# LightCraft design refresh · Fable notes

Claude Fable 5.1, high effort, 2026-10-08. Design exploration only: one self-contained mockup (`mockup.html`), no source edits, no build, no app launch, no network.

## Open it

`mockup.html` runs from `file://`. Assets are referenced relatively and must stay where they are:

| Asset | Relative path | Use |
|---|---|---|
| Inter Regular / Medium / SemiBold | `../../../source/assets/fonts/Inter-*.ttf` | UI font (OFL; `source/assets/ATTRIBUTION.md`) |
| `grid-demo.jpg` | `../../../source/docs/images/` | 15 real procedural-scene renders from LightCraft's own egui grid screenshot, cropped as thumbnails |
| `grid-pd.jpg` | same | 4 public-domain photos as thumbnails |
| `presets-mixer.jpg`, `hero-tetons.jpg`, `info-earthrise.jpg`, `grading-migrant-mother.jpg`, `ba-blue-marble.jpg` | same | loupe-size sources for Lavender rows, The Tetons, Earthrise, Migrant Mother, The Blue Marble |

URL parameters (all optional):

| Parameter | Values | Default |
|---|---|---|
| `view` | `library` · `develop` | `library` |
| `theme` | `light` · `dark` | `dark` |
| `platform` | `macos` · `windows` | `macos` |
| `frame` | `fill` · `1600x1000` · `1280x800` · `1600x788` · any `WxH` | `fill` (the window is the viewport) |
| `panel` | `edit` · `crop` · `remove` · `mask` · `redeye` · `presets` · `versions` · `history` · `keywords` · `info` | `edit` in Develop, `info` in Library |
| `photo` | photo id 1–19 | `4` (Lavender rows) |
| `source` | `all` · `recent` · `picks` · `deleted` · `missing` · `date` · `album:<id>` | `all` |
| `sidebar` | `0` · `1` | open in Library, collapsed rail in Develop |
| `inspector`, `filmstrip` | `0` to hide | shown |
| `modal` | `import` · `export` · `help` | none |
| `error` | `1` shows the unreadable-file state | off |
| `motion` | `reduce` forces reduced motion | follows the OS |
| `harness` | `0` hides the capture pill | shown |
| `names` | `1` always shows file names on cells | hover / selection only |

For the 1600×788 comparison against the current screenshots use `?frame=1600x788&harness=0`; everything scales inside the frame without clipping.

## The design in one paragraph

RightKit's fused shell stays exactly as published (chrome surface for titlebar and sidebar, plane with the 20 px top-left radius, active nav row joined with the 12 px concave curves, palette trigger, wordmark + appearance foot). LightCraft adds one thing to that language: a right-hand tool rail on the same chrome surface whose active tool fuses into the inspector the same way the sidebar's active row fuses into the plane. The titlebar holds a single Library | Develop segmented control (G / D). Library is a justified photo grid with search, chips, and an Info panel for the selected photo. Develop is a neutral stage, a bottom stage bar, a stable filmstrip that always names its source and filter, and a 320 px inspector with a measured histogram and long two-line sliders. Switching views changes only the centre; sidebar, rail and inspector keep their places.

## Decisions and why

**One view switch, in the titlebar.** The current app has view tabs in the stage top bar (Detail / Compare / Survey / Reference / People), a Library title in the titlebar, and grid buttons in both the header and footer. The segmented control is the RightKit idiom for the active view title, so Library | Develop lives there. Compare / Survey / Reference are stage modes inside Develop (bottom stage bar), because they are ways of looking at the selection, not workspaces.

**One tool rail, not two.** `StageWorkspace` renders a 74 px labelled tool strip and `Inspector` renders a second strip of the same tools above the panel. The design keeps one 44 px icon rail on the chrome surface at the far right. Tooltips carry label and key. The rail is identical in both views; in Library the Info panel is active by default, and picking an edit tool opens Develop with that panel (the parent's earlier decision). The rail's active item uses the RightKit `FuseCurves` shape mirrored to join the inspector surface. When the inspector is collapsed (⌘]) the active tool becomes a quiet accent pill, because there is no surface to fuse into.

**Photo area first.** Develop collapses the sidebar to the 48 px rail by default (the sidebar state is remembered per view, so Library gets it back). Inspector 320 px, rail 44 px, stage bar 40 px, filmstrip 104 px (30 px context row + 66 px thumbnails) are the only fixed chrome. At 1600×1000 the stage is about 1180×800; at 1280×800 about 860×600. Sidebar and inspector collapse independently; the inspector drags 260–520 px and double-click resets to 320.

**Filmstrip keeps the source and the filter.** Its first row is a context chip (`All Photos · ★3+ · Picks`) and a position readout (`4 of 11 · 3 selected`). The chip returns to that exact source in the Library. The filmstrip list is the same `visible()` list as the grid, so a filter set in Library is what the arrow keys walk in Develop. Collapsing the filmstrip (F) keeps the 30 px context row so the context is never lost.

**Edits never leave the photo.** Edit state, history and crop live per photo id. Stepping to another photo and back restores sliders, history and undo stack. The stage HUD shows `Saving edits…` then `Saved` after each gesture, mirroring the engine's auto-persist and the sidebar status row, so there is no "did it take?" moment.

**Sliders that do not flash.** Each control is one `<input type=range>` plus a text value and a reset. During a gesture the row owns its value; nothing re-renders the row from outside until the gesture ends (pointer up, blur, or 400 ms after the last key). The number turns accent when modified, the section header gets a dot, and the whole row never changes height. This is the UI contract for the real begin / set / end interaction protocol: the snapshot must not overwrite a row that is mid-gesture. Two-line rows (label and value on top, long track below) give a 290 px track at the default width, which is what makes fine exposure moves possible.

**Beginners first, depth reachable.** Default inspector shows histogram, Auto / B&W / Reset all, Profile, White balance, Light and Color open, Effects and Detail collapsed, and a "More adjustments" disclosure for Optics, Geometry and Calibration. Tone curve, Color mixer and Color grading are sub-sections inside Light and Color rather than separate panels. ⌘1–⌘4 toggle the four main sections.

**Professional histogram.** RGB channels drawn as filled outlines over a luma fill, with shadow and highlight clipping triangles at the corners that light up when clipping is measured, an exposure-facts row, and a sample count. When the browser cannot read pixels (tainted canvas from `file://`) the panel says so instead of pretending.

**Library grid.** Justified rows (equal row height, true aspect) replace the current column grid, which left large gaps around portraits. Rating stars always sit bottom-left with a drop shadow; file name appears on hover, selection, or via the sort menu's "Show file names". Pick / reject / label / edited / missing badges sit top-left. Selection is a 2 px accent ring, the active photo adds an inner white line. Thumbnail size and Photo / Square grid live in the header; the footer is gone, and its Copy / Paste settings moved to the context menu, the sort menu and the Info panel.

**Import that is never silent.** Import opens a review sheet immediately. Picking a source shows "Waiting for the system picker…" (the native picker is outside the webview and the user needs to know we are waiting on it), then "Reviewing … checking for duplicates" with an indeterminate bar and Cancel, then a candidate list where duplicates are greyed and unchecked with the reason. Import closes the sheet and reports in three places at once: titlebar pill with cancel, sidebar status row, and a progress toast; the grid shows shimmering placeholder cells where the photos will land. Completion is a toast with "Show".

**Errors and empties are designed, not accidental.** Empty states use RightKit `EmptyState` with a next step (Import, Clear filters). A missing original shows a badge on the cell, a banner in Missing Photos, a warning HUD on the stage with Locate, and a line in Info. An unreadable file shows an error toast with Retry, a banner above the grid, and a bad dot in the sidebar status.

**Stage modes.** Detail is the only mode rendered in the mockup. Compare, Survey and Reference exist in the mode control and explain what the engine would lay out. People is reachable from the palette and explains that it needs face data.

**Theme.** Both themes use RightKit's tokens unchanged; LightCraft only sets brand `#0165dd` / `#4d9bff`, the opaque focus ring, and neutral photo surfaces: stage `#1c1c1c` dark / `#4a4a4d` light, grid `#121214` / `#f3efe7`. Nothing warm touches the photo.

## Self-critique (adversarial pass)

- **Does the default editor maximise photo area?** Mostly. The inspector is 320 px by default, 20 px wider than today, because two-line sliders need it. At 1280×800 with the inspector open the stage is ~860 px wide, which meets the 360 px invariant comfortably but is not generous. ⌘] gives the full width in one keystroke; the design does not auto-collapse the inspector at narrow widths, which a real implementation should consider at < 1100 px.
- **Are duplicate controls removed?** Yes: one view switch, one tool rail, one grid-style control, one rating control per view. The rating pill in the stage bar and the stars in the Info panel both exist, but they serve different views; in Develop the Info panel also shows stars, which is a mild duplicate kept for metadata editing.
- **Can beginners start while advanced controls stay reachable?** Light and Color are open with the common six; Optics / Geometry / Calibration are behind one disclosure. Risk: the Tone curve and Mixer sub-sections add visual nesting that a power user might find slower than top-level panels. ⌘-number shortcuts and persistent open state mitigate this.
- **Does the filmstrip retain source and filter context?** Yes, explicitly, including when collapsed.
- **Does changing photo lose unsaved edit context?** No. Per-photo state, visible save status, and undo per photo.
- **Are resize and loading states stable?** Draft preview is shown immediately from the thumbnail crop and the full preview replaces it in place without a layout change; the HUD reports "Rendering preview…" only while an image is actually decoding. Justified rows re-flow on resize; cells do not change aspect. Weak spot: re-flowing the grid on resize re-renders the DOM, which drops grid focus in the mockup; the real grid should keep its virtualised nodes.
- **Is anything a dashboard?** No cards, no KPIs. The only numbers outside the photo are counts next to sources and the position readout.
- **Known rough edges of the mockup itself:** stage modes other than Detail are not drawn; thumbnails are ~600 px crops, so Develop shows a soft draft for scenes without a loupe-size source; the Info preview in Library uses the same crop; the "Waiting for the system picker" phase is a timed stand-in because there is no native picker here; sort by date does not group by month; the Lightroom catalog flow is only a palette entry.

## What is simulated

- Slider motion applies CSS `brightness / contrast / saturate` plus tint and vignette overlays to the preview for feel only. Values are not LightCraft pipeline output.
- The histogram and the clipping overlay are computed from the decoded preview pixels in the browser when `getImageData` is allowed; on `file://` most browsers taint the canvas and the panel then shows an "Illustrative" placeholder and clipping explains why it is unavailable. Neither is engine output.
- Import, export, save status, "Rendering preview…" (tied to the real image decode in the browser), duplicates, candidate paths, and the two "new" photos are fixtures with timers. The import adds two real renders that were held back from the initial library so that nothing is invented.
- Presets set slider values; Auto sets fixed values; versions, keyword sets, profiles, album membership and metadata presets are static lists.
- Native actions (folder picker, reveal in Finder, fullscreen) show a toast describing what the real app does.
- Crop handles drag and persist; straighten rotates the overlay only. Masks render as a red overlay for the selected mask; brush strokes are not painted. Remove places circles.

## Acceptance at the two viewports

Verified by reading the layout rules, not by rendering (no browser was available to this session):

- **1600×1000, both views, both themes:** titlebar 52, sidebar 268 (rail 48 in Develop), rail 44, inspector 320. Library grid area ≈ 920×790 with the Info panel open; Develop stage ≈ 1180×800 above a 104 px filmstrip and 40 px stage bar. No horizontal scrolling anywhere; the filter bar and stage bar wrap before overflowing.
- **1280×800:** Library grid ≈ 600×590 with Info open (3 cells per row at the default 200 px row height), ≈ 920×590 with the inspector collapsed; Develop stage ≈ 860×600. The thumbnail-size slider hides below 1100 px plane width to keep the header on one line. Both panels can be collapsed with ⌘\ and ⌘].
- Keyboard: Tab reaches the sidebar search, nav (roving arrows), view switch, header controls, grid (arrows / Home / End / Enter), rail, inspector grip (←/→ resize), every slider and value field, and the filmstrip (←/→). Every focusable element shows an opaque 2 px ring with a plane-coloured halo.
- Reduced motion disables transitions, shimmer, spinners' rotation and smooth scrolling (`prefers-reduced-motion` or `?motion=reduce`).

Please confirm with real renders; the geometry above is arithmetic from the stylesheet.

## Improvement priorities for implementation

1. Replace the inspector's raw rows with one shared slider component that owns its draft value for the gesture lifetime (begin / set / end) and ignores snapshot updates while active. This removes the flash the user reported regardless of engine timing.
2. Import: open the review sheet before the preview resolves and show the three phases (picker, reviewing, submitting) with cancel; publish progress to titlebar, sidebar status and a toast from one job state.
3. Single tool rail replacing `StageWorkspace.ToolStrip` and `Inspector.ToolStrip`; Library/Develop segmented control in the titlebar; drop the Library footer.
4. Justified grid layout and overlay badges; file names on hover / selection.
5. Filmstrip context row bound to the same view slice as the grid.
6. Per-view sidebar memory (collapsed in Develop by default).
7. Histogram with measured clipping indicators driven by the engine's histogram payload (it already arrives with every preview).
8. Stage HUD for preview build and save status from `status.previewBuild` and `status.unsaved`.

## Component reuse (from `@rightkit/app-shell` 0.2.1)

| Need | RightKit piece | Notes |
|---|---|---|
| Frame, titlebar, sidebar, plane | `AppShell` (or `AppFrame` + `Titlebar` + `Plane` + `NavList` for the right rail) | The rail needs the custom composition path the README describes, because `AppShell` has no right-rail slot |
| Fused active rows | `FuseCurves` | Reused for the rail with `transform: scaleX(-1)` |
| Library / Develop switch, grid style, flags, modes | `SegmentedControl` | `role="radiogroup"` as published |
| Buttons, icon buttons, inputs, toggles, badges, pills | `Button`, `IconButton`, `Input`, `Toggle`, `Badge`, `Pill`, `StatusRow` | `IconButton` requires `label`, which gives the rail its names |
| Empty states | `EmptyState` | Grid empties, missing originals |
| Import sheet, export, help | `ModalShell` / `Dialog` | Focus trap, Escape, backdrop, opener restore, inert background (audit gap 2) |
| Context menu, sort menu, filters, appearance | `ContextMenu` / `useContextMenu` | Arrow keys, Home / End, Escape, scrim, focus restore (audit gap 4) |
| Palette, shortcut help | `CommandPalette`, `ShortcutHelp`, `useShortcuts` | Single registry; the `?` help should be RightKit's (audit gap 3) |
| Toasts | `toast()`, `ToastViewport` | Progress toasts carry a Cancel action |
| Theme | `createThemeStore`, `AppearanceMenu` | Unchanged |

LightCraft-owned: grid and filmstrip virtualisation, slider component, histogram, stage, crop / mask overlays, inspector sections.

## Sources read

- Screenshots: `review/design-refresh/current-library.png`, `current-develop.png`.
- Source: `apps/lightcraft-desktop/web/src/{App.tsx, library/LibraryWorkspace.tsx, library/Filmstrip.tsx, stage/StageWorkspace.tsx, inspector/Inspector.tsx, app.css, library/library.css, inspector/Inspector.css, stage/StageWorkspace.css, types.ts, dialogs/DialogHost.tsx (ImportDialog), desktop.tsx (import routing)}`; `crates/develop/src/controls.rs` (control ids, ranges, defaults, track kinds); `crates/scenes/src/{lib.rs, paint.rs}` (scene names, metadata).
- RightKit: `node_modules/@rightkit/app-shell/{README.md, shell.css, suite-aliases.css, dist/react/{layout.js, layout.d.ts, ui.d.ts, overlays.d.ts, toasts.d.ts}}`.
- Audit and prior reviews: `review/rightkit-component-audit.md`, `review/claude/{adversarial-review.md, refined-plan.md, mockup-readme.md}`; the historical `review/claude/mockup.html` was read for asset paths only and not reused.
- ScrapeRight: `ScrapeRight `src/App.tsx``, `components/DesktopWorkspace.tsx`, first part of `components/ClaudeDesktop.css` (sidebar rail, resize handle, status copy patterns).

## Assumptions

- The RightKit shell CSS is reproduced in the mockup with the same class names and token values so the structure maps one-to-one; the real app imports the package stylesheet.
- Thumbnail crops are ±4 px; the egui screenshots carry baked-in star badges bottom-left, so each tile keeps its top 90 % and drops the left 5 %.
- Temperature is shown in kelvin per `controls.rs` (2000–50000, default 6500); the current React build shows the raw minimum because the snapshot had not loaded, which is a bug, not a design.
- Photo file names follow the current app's `LC0xxxx.JPG` pattern; the four archive scans keep their real names.
- The Library Info panel is open by default on the assumption that culling benefits from metadata and rating controls; if the parent prefers a wider grid, `?inspector=0` is the alternative default.

## Attribution

- Inter: Rasmus Andersson, SIL OFL 1.1 (`source/assets/fonts/OFL-Inter.txt`).
- Procedural demo scenes: LightCraft contributors, generated by `crates/scenes` at runtime; shown here as crops of LightCraft's own documentation screenshots (`docs/images/grid-demo.jpg`, `presets-mixer.jpg`), MIT OR Apache-2.0.
- "The Tetons and the Snake River", Ansel Adams, 1942, U.S. National Archives, public domain (`hero-tetons.jpg`, `grid-pd.jpg`).
- "Earthrise", NASA / Bill Anders, Apollo 8, 1968, public domain (`info-earthrise.jpg`, `grid-pd.jpg`).
- "Migrant Mother", Dorothea Lange, 1936, Library of Congress, public domain (`grading-migrant-mother.jpg`, `grid-pd.jpg`).
- "The Blue Marble", NASA, Apollo 17, 1972, public domain (`ba-blue-marble.jpg`, `grid-pd.jpg`).
- Icons: original line icons drawn in this mockup. No Adobe assets, fonts, iconography or sample photos were used or inspected.
