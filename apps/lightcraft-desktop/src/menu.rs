//! Stable native menu IDs shared with engine command metadata.

#![forbid(unsafe_code)]

use rightkit_shell::{MenuEntry, MenuGroup, MenuSpec};

fn group(label: &str, entries: &[(&str, &str, Option<&str>)]) -> MenuGroup {
    MenuGroup {
        label: label.into(),
        entries: entries
            .iter()
            .map(|(id, title, accelerator)| accelerator.map_or_else(|| MenuEntry::item(id, title), |key| MenuEntry::accel(id, title, key)))
            .collect(),
    }
}

/// Native menu contains IDs used by existing `menus.rs`; frontend forwards clicks through
/// `rightkit-shell://menu` so command routing remains authoritative in engine/session.
pub fn spec() -> MenuSpec {
    MenuSpec {
        app_name: "LightCraft".into(),
        settings: Some(("app.settings".into(), "Cmd+,".into())),
        groups: vec![
            group(
                "File",
                &[
                    ("file.addPhotos", "Import Photos…", Some("Cmd+Shift+I")),
                    ("file.addFolder", "Import from Folder…", None),
                    ("file.importLightroom", "Import Lightroom Catalog…", None),
                    ("file.addFromDevice", "Import from Device", None),
                    ("file.findMissing", "Find Missing Photos…", None),
                    ("file.backupLibrary", "Back Up Library…", None),
                    ("file.restoreLibrary", "Restore Library from Backup…", None),
                    ("file.importPresets", "Import Profiles & Presets…", None),
                    ("file.exportPresets", "Export Presets…", None),
                    ("file.importCurvePresets", "Import Point Curve Presets…", None),
                    ("file.exportCurvePresets", "Export Point Curve Presets…", None),
                    ("dialog.export", "Export…", None),
                    ("app.openLibrary", "Open Library…", None),
                    ("app.quit", "Quit LightCraft", Some("Cmd+Q")),
                ],
            ),
            group(
                "Edit",
                &[("edit.undo", "Undo", Some("Cmd+Z")), ("edit.redo", "Redo", Some("Cmd+Shift+Z")), ("view.focusSearch", "Find…", Some("Cmd+F"))],
            ),
            group(
                "View",
                &[
                    ("view.photoGrid", "Photo Grid", None),
                    ("view.squareGrid", "Square Grid", None),
                    ("view.gridToggle", "Grid", Some("G")),
                    ("view.detail", "Detail", Some("D")),
                    ("view.compare", "Compare", Some("Shift+C")),
                    ("view.survey", "Survey", Some("N")),
                    ("view.people", "People", None),
                    ("view.reference", "Reference View", Some("Shift+R")),
                    ("view.filmstrip", "Filmstrip", Some("/")),
                    ("view.beforeAfter", "Compare Before and After", Some("Y")),
                    ("view.beforeAfterSplit", "Before/After Split", Some("Shift+Y")),
                    ("view.beforeAfterTopBottom", "Before/After Top/Bottom", Some("Alt+Y")),
                    ("view.beforeAfterSplitTopBottom", "Before/After Split Top/Bottom", Some("Alt+Shift+Y")),
                    ("view.showOriginal", "Show Original", Some("\\")),
                    ("view.zoomFit", "Zoom to Fit", Some("Cmd+0")),
                    ("view.zoom100", "Zoom 100%", Some("Cmd+Alt+0")),
                    ("view.zoomToggle", "Toggle Zoom", Some("Z")),
                    ("view.zoomIn", "Zoom In", Some("Cmd+=")),
                    ("view.zoomOut", "Zoom Out", Some("Cmd+-")),
                    ("view.clipping", "Show Clipping", Some("J")),
                    ("view.softProof", "Soft Proofing", Some("S")),
                    ("view.histogram", "Histogram", Some("Cmd+Shift+H")),
                    ("view.maskOverlay", "Show Mask Overlay", Some("O")),
                    ("view.filterBar", "Filter Bar", Some("Shift+F")),
                    ("view.navigator", "Navigator", None),
                    ("view.enterFullScreen", "Enter Full Screen", Some("Cmd+Shift+F")),
                    ("view.infoOverlay", "Cycle Info Overlay", Some("Cmd+I")),
                ],
            ),
            group(
                "Photo",
                &[
                    ("photo.editInExternal", "Edit in External Editor", Some("Cmd+Shift+E")),
                    ("photo.locate", "Locate Missing File…", None),
                    ("dialog.rename", "Rename Photos…", Some("F2")),
                    ("dialog.allMetadata", "All Metadata…", None),
                ],
            ),
            group(
                "Window",
                &[
                    ("view.secondWindow", "Second Window", Some("Cmd+F11")),
                    ("panel.edit", "Edit", Some("E")),
                    ("panel.crop", "Crop & Rotate", Some("C")),
                    ("panel.masking", "Masking", Some("M")),
                    ("panel.presets", "Presets", Some("Shift+P")),
                    ("panel.info", "Info", Some("I")),
                    ("panel.keywords", "Keywords", Some("K")),
                    ("panel.versions", "Versions", Some("Shift+V")),
                    ("panel.activity", "History", None),
                    ("panel.close", "Close Panel", None),
                    ("tool.brush", "Brush", Some("B")),
                    ("tool.linear", "Linear Gradient", Some("L")),
                    ("tool.radial", "Radial Gradient", Some("R")),
                    ("tool.wbPicker", "White Balance Selector", Some("W")),
                ],
            ),
        ],
        help: vec![
            MenuEntry::item("app.about", "About LightCraft"),
            MenuEntry::item("app.help", "LightCraft Help"),
            MenuEntry::item("app.github", "LightCraft on GitHub"),
        ],
    }
}
