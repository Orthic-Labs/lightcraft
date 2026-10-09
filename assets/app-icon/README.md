# Ember app icon

Original Orthic Labs flame artwork; canonical source: `ember.svg`. Charcoal tile (`#161618`), amber flame (`#ffad57`) & cream core (`#fff0d9`), 512 × 512 viewBox, corner radius 112. No upstream mark is used.

- `ember-1024.png`: documentation/store artwork.
- `ember-macos-512.png`, `ember.icns`: macOS transparent-margin variants.
- `ember.ico`: Windows 16–256 px icon family.
- `hicolor/<size>/apps/com.orthiclabs.ember.png`: Linux 16–512 px icons.
- `hicolor/scalable/apps/com.orthiclabs.ember.svg`: scalable source.

Edit SVG, then run `packaging/icons.sh` with Python 3 & Pillow. `packaging/render-icons.py` supports only this source’s SVG path vocabulary, uses supersampling & writes committed assets without building an application. Licence: MIT OR Apache-2.0; see `LICENSE.txt`, root licence files & `assets/ATTRIBUTION.md`.
