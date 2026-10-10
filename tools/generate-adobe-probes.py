"""Original, deterministic numerical inputs for black-box photo-editor study.

Not photographic validation. No vendor data, profiles or camera matrices.
"""
import argparse
from array import array
from pathlib import Path
import hashlib
import json
import math
import random
import struct
import sys
from PIL import Image

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--output", required=True, type=Path, help="Directory outside repository")
args = parser.parse_args()
ROOT = args.output.expanduser().resolve()
repo = Path(__file__).resolve().parent.parent
if ROOT == repo or repo in ROOT.parents:
    raise SystemExit("Generated media must stay outside repository")
OUT = ROOT / "inputs"
if OUT.exists():
    raise SystemExit("Output inputs directory already exists; choose fresh output")
OUT.mkdir(parents=True)
records = []

def receipt(path, kind, **params):
    records.append(dict(name=path.name, sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                        bytes=path.stat().st_size, kind=kind, parameters=params))

def tiff(name, values, **params):
    im = Image.new("RGB", (512, 512))
    im.putdata(values)
    path = OUT / (name + ".tif")
    im.save(path, compression="raw")
    receipt(path, "lossless-rgb-tiff", **params)

for value in (32, 128, 224):
    tiff(f"flat-{value:03}", [(value,)*3]*(512*512), rgb=value)
ramp = [(x//2,)*3 for y in range(512) for x in range(512)]
tiff("ramp-spatial", ramp, histogram="each RGB gray 0..255 occurs 1024 times")
shuffled = ramp.copy()
random.Random(20261010).shuffle(shuffled)
tiff("ramp-shuffled", shuffled, histogram="identical to ramp-spatial", seed=20261010)
hist_a = Image.new("RGB", (512, 512)); hist_a.putdata(ramp)
hist_b = Image.new("RGB", (512, 512)); hist_b.putdata(shuffled)
assert hist_a.histogram() == hist_b.histogram()
tiff("color-patches", [((x//64)*36, (y//64)*36, ((x//64+y//64)%8)*36)
                       for y in range(512) for x in range(512)], purpose="saturation & chroma response")

def write_dng(name, noisy):
    w = h = 1024
    rng = random.Random(20261010)
    samples = array("H")
    for y in range(h):
        for x in range(w):
            # Flat patches, smooth ramp, sinusoidal detail & one sharp edge.
            if y < 256:
                v = (0.02, 0.08, 0.18, 0.5)[x//256]
            elif y < 512:
                v = 0.02 + 0.7*x/(w-1)
            elif y < 768:
                v = 0.18 + 0.035*math.sin(x*math.pi/4)
            else:
                v = 0.06 if x < w//2 else 0.5
            # Synthetic sensor is an identity XYZ sensor; no camera calibration.
            # Neutral D65 channel ratios for R/G/B-labelled CFA samples.
            channel = ((0, 1), (1, 2))[y%2][x%2]
            v *= (0.95047, 1.0, 1.08883)[channel]
            if noisy:
                v += rng.gauss(0, math.sqrt(max(v, 0)*0.00012 + 0.000004))
            samples.append(round(512 + max(0, min(1, v))*15871))
    if sys.byteorder != "little":
        samples.byteswap()
    pixels = samples.tobytes()
    tags = {}
    def short(tag, vals): tags[tag] = (3, len(vals), struct.pack("<"+"H"*len(vals), *vals))
    def long(tag, vals): tags[tag] = (4, len(vals), struct.pack("<"+"I"*len(vals), *vals))
    def byte(tag, vals): tags[tag] = (1, len(vals), bytes(vals))
    def ascii_(tag, text):
        b = text.encode("ascii")+b"\0"; tags[tag] = (2,len(b),b)
    def rational(tag, pairs, signed=False):
        tags[tag] = (10 if signed else 5, len(pairs),
                     b"".join(struct.pack("<ii" if signed else "<II", *p) for p in pairs))
    long(254,[0]); long(256,[w]); long(257,[h]); short(258,[16]); short(259,[1])
    short(262,[32803]); ascii_(271,"Orthic Original"); ascii_(272,"Synthetic XYZ Bayer")
    long(273,[0]); short(274,[1]); short(277,[1]); long(278,[h]); long(279,[len(pixels)])
    short(284,[1]); ascii_(305,"Original numerical probe generator")
    short(33421,[2,2]); byte(33422,[0,1,1,2]); byte(50706,[1,4,0,0]); byte(50707,[1,3,0,0])
    ascii_(50708,"Orthic Original Synthetic XYZ Bayer")
    byte(50710,[0,1,2]); short(50711,[1]); short(50713,[1,1]); rational(50714,[(512,1)])
    long(50717,[16383]); long(50719,[0,0]); long(50720,[w,h])
    rational(50721,[(1 if i in (0,4,8) else 0,1) for i in range(9)],signed=True)
    rational(50728,[(95047,100000),(1,1),(108883,100000)])
    rational(50730,[(0,1)],signed=True); short(50778,[21]); long(50829,[0,0,h,w])
    keys = sorted(tags)
    offset = 8 + 2 + 12*len(keys) + 4
    extra = bytearray(); offsets={}
    for tag in keys:
        data=tags[tag][2]
        if len(data)>4:
            if (offset+len(extra))%2: extra.append(0)
            offsets[tag] = offset+len(extra); extra.extend(data)
    if (offset+len(extra))%2: extra.append(0)
    pixel_offset=offset+len(extra)
    long(273,[pixel_offset])
    entries=bytearray()
    for tag in keys:
        type_, count, data=tags[tag]
        value=struct.pack("<I", offsets[tag]) if len(data)>4 else data.ljust(4,b"\0")
        entries.extend(struct.pack("<HHI",tag,type_,count)+value)
    path=OUT/(name+".dng")
    path.write_bytes(b"II"+struct.pack("<HIH",42,8,len(keys))+entries+struct.pack("<I",0)+extra+pixels)
    receipt(path,"original-synthetic-bayer-dng", width=w,height=h,black=512,white=16383,
            cfa="RGGB", color_matrix="identity XYZ (original synthetic sensor)",noisy=noisy,seed=20261010)

write_dng("bayer-clean", False)
write_dng("bayer-noisy", True)
(ROOT/"input-receipts.json").write_text(json.dumps(dict(schema=1,source="original procedural work",
    generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),inputs=records), indent=2)+"\n")
print(json.dumps(dict(inputs=len(records), histogram_pair_equal=True,output=str(OUT))))
