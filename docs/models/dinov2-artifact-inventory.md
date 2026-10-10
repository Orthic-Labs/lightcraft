# DINOv2 ViT-S/14 artifact inventory

This inventory covers exact official artifact held outside repository at
`/tmp/weights_aesthetic_dinov2/dinov2_vits14_pretrain.pth`.

| Field | Evidence |
| --- | --- |
| URL | `https://dl.fbaipublicfiles.com/dinov2/dinov2_vits14/dinov2_vits14_pretrain.pth` |
| Source revision | DINOv2 repository commit `7764ea0f912e53c92e82eb78a2a1631e92725fc8` |
| Size | `88,283,115` bytes |
| SHA-256 | `b938bf1bc15cd2ec0feacfe3a1bb553fe8ea9ca46a7e1d8d00217f29aef60cd9` |
| Container | PyTorch ZIP serialization; 177 stored entries: 175 storages + `data.pkl` + `version` |
| Pickle | `vits14_pretrain/data.pkl`, 21,829 bytes |
| Tensor payloads | 175 `torch FloatStorage` entries; 88,226,304 bytes total |
| Payload evidence | Static `FloatStorage` records with contiguous ranges; loader reads ranges as little-endian F32 only after exact full-byte SHA-256 verification |

## Extraction evidence

Static inspection used Python `zipfile` metadata plus `pickletools.genops` on
`data.pkl`. No `pickle.load`, `torch.load`, import of pickle globals, model
construction, tensor decode, or inference was performed. For each
`BINPERSID` tensor record, extraction recorded state-dict key, storage ID,
storage element count, shape, stride, then resolved storage ID to its ZIP local
header. Offset = local-header offset + 30 + filename length + extra-field length.

Checks passed on 2026-10-10:

- 175 tensor records, 175 unique keys, 175 unique storage IDs.
- Every storage global is `torch FloatStorage`; every ZIP entry uses stored
  compression.
- Every storage offset is zero; every stride is contiguous row-major stride.
- Every shape product equals storage element count; each `byte_len` equals
  element count × 4.
- Every payload range is inside 88,283,115-byte artifact, with no overlap.
- Remaining two ZIP entries are `data.pkl` & `version`; their bytes plus tensor
  payloads equal ZIP uncompressed total `88,248,135` bytes. This artifact has no
  `byteorder` member.

`crates/segment/src/dinov2_inventory.rs` preserves state-dict order as a
descriptor table; no artifact bytes are included. Sibling
`crates/segment/src/dinov2_artifact.rs` verifies full input bytes against exact
size + SHA-256 before using ranges. It interprets recorded ranges as little-endian
F32, without parsing ZIP or pickle metadata at runtime.

## Provenance & license

DINOv2 source, model card, & repository license are Apache-2.0:

- [Model card](https://github.com/facebookresearch/dinov2/blob/7764ea0f912e53c92e82eb78a2a1631e92725fc8/MODEL_CARD.md)
- [Repository license](https://github.com/facebookresearch/dinov2/blob/7764ea0f912e53c92e82eb78a2a1631e92725fc8/LICENSE)

No separate binary notice was found in pinned sources. LVD-142M training data
is not redistributed. Checkpoint redistribution still requires license review
before packaging; current artifact remains scratch-only.

## Bounds

Inventory qualifies byte identity, container shape, storage layout, static
`FloatStorage`/range evidence, & key completeness only. Runtime little-endian
F32 interpretation is fixed by loader contract. It does not qualify
conversion parity, preprocessing, embedding quality, culling accuracy, memory,
or platform latency. Runtime loader must reverify `MODEL_BYTES` plus
`MODEL_SHA256` on exact bytes, bound counts, lengths, offsets, shapes, & ranges,
then map only these 175 keys to F32 tensors; exact full-byte verification makes
ZIP/pickle parsing unnecessary at runtime.
