# Ember macOS installer window

Finder content is 660 × 400 pt. Original Ember artwork occupies left panel; light area preserves readable Finder labels. App icon: `Ember.app` at (326, 205); Applications link: (574, 205); icon size 128.

`background.svg` is source artwork. `generate.py` renders its deliberately small SVG vocabulary with repository Inter fonts & original Ember icon, then writes 1x/2x `background.tiff` (72/144 dpi) & deterministic `dmg-layout.DS_Store`. Volume name is `Ember`, matching `package.sh`; version belongs in artifact filename.

Regenerate assets with isolated Python tooling:

```sh
python3 -m venv /tmp/ember-asset-env
/tmp/ember-asset-env/bin/pip install pillow==12.1.1 ds_store==1.3.3 mac_alias==2.2.3
/tmp/ember-asset-env/bin/python packaging/macos/dmg/generate.py
```

This generates artwork/layout only. No native build, Finder automation or app launch is needed. Fixed alias/profile metadata avoids local paths, timestamps & volume UUIDs. Generator’s layout/alias code derives from upstream work by @XusBadia; artwork is original Orthic Labs work. See `assets/ATTRIBUTION.md`.
