//! Deterministic native qualification inputs.
//!
//! These builders deliberately create real media and a small, read-only SQLite catalog. They
//! avoid checked-in binary fixtures while keeping native qualification reproducible on every host.

#![cfg(any(target_os = "macos", target_os = "windows"))]

use std::path::{Path, PathBuf};

use lightcraft_tiff::tags as t;
use lightcraft_tiff::{IfdBuilder, ImageData, TiffWriter, Value};

const ARW_WIDTH: u32 = 32;
const ARW_HEIGHT: u32 = 16;
const PNG_WIDTH: u32 = 96;
const PNG_HEIGHT: u32 = 64;
const MAX_DIMENSION: u32 = 4096;
const SONY_BLACK_LEVEL: u16 = 0x7310;
const SONY_WB_RGGB: u16 = 0x7313;

/// Paths written by [`write_fixture_inputs`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureInputs {
    pub root: PathBuf,
    pub arw: PathBuf,
    pub png: PathBuf,
    pub catalog: PathBuf,
}

/// Write one real Sony ARW, one sRGB PNG, and one Lightroom catalog that references both.
///
/// `root` is created when missing. The catalog points at `root/Photos/synthetic-sonya-01.arw`,
/// so callers can import it without rewriting paths or staging private fixture bytes.
pub fn write_fixture_inputs(root: &Path) -> Result<FixtureInputs, String> {
    std::fs::create_dir_all(root).map_err(|e| format!("create fixture root: {e}"))?;
    let root = std::fs::canonicalize(root).map_err(|e| format!("canonicalize fixture root: {e}"))?;
    let photos = root.join("Photos");
    std::fs::create_dir_all(&photos).map_err(|e| format!("create fixture photos: {e}"))?;

    let arw = photos.join("synthetic-sonya-01.arw");
    let png = photos.join("procedural-rgb-01.png");
    let catalog = root.join("synthetic-lightroom.lrcat");
    std::fs::write(&arw, synthetic_sony_arw(ARW_WIDTH, ARW_HEIGHT)?).map_err(|e| format!("write ARW: {e}"))?;
    std::fs::write(&png, procedural_png(PNG_WIDTH, PNG_HEIGHT)?).map_err(|e| format!("write PNG: {e}"))?;
    let file_name = arw.file_name().and_then(|n| n.to_str()).ok_or_else(|| "ARW fixture name is not UTF-8".to_string())?;
    std::fs::write(&catalog, synthetic_lightroom_catalog(&root, file_name)?).map_err(|e| format!("write Lightroom catalog: {e}"))?;
    Ok(FixtureInputs { root, arw, png, catalog })
}

/// Build a little-endian TIFF-shaped Sony ARW with an uncompressed 14-bit RGGB sensor plane.
///
/// Samples use a stable gradient instead of a text placeholder, which exercises the actual raw
/// probe, CFA decode, white balance, demosaic, and renderer paths.
pub fn synthetic_sony_arw(width: u32, height: u32) -> Result<Vec<u8>, String> {
    validate_dimensions(width, height)?;
    let sample_count = width.checked_mul(height).ok_or_else(|| "ARW dimensions overflow".to_string())? as usize;
    let mut strip = Vec::with_capacity(sample_count.saturating_mul(2));
    for y in 0..height {
        for x in 0..width {
            let value = 512u16 + (((x * 173 + y * 257 + (x ^ y) * 31) % 13_500) as u16);
            strip.extend_from_slice(&value.to_le_bytes());
        }
    }

    let mut raw = IfdBuilder::new();
    raw.set(t::NEW_SUBFILE_TYPE, Value::Long(vec![0]));
    raw.set(t::IMAGE_WIDTH, Value::Long(vec![width]));
    raw.set(t::IMAGE_LENGTH, Value::Long(vec![height]));
    raw.set(t::BITS_PER_SAMPLE, Value::Short(vec![14]));
    raw.set(t::COMPRESSION, Value::Short(vec![1]));
    raw.set(t::PHOTOMETRIC, Value::Short(vec![t::photometric::CFA]));
    raw.set(t::SAMPLES_PER_PIXEL, Value::Short(vec![1]));
    raw.set(t::PLANAR_CONFIGURATION, Value::Short(vec![1]));
    raw.set(t::CFA_REPEAT_PATTERN_DIM, Value::Short(vec![2, 2]));
    raw.set(t::CFA_PATTERN_EP, Value::Byte(vec![0, 1, 1, 2]));
    raw.set(SONY_BLACK_LEVEL, Value::Short(vec![512, 512, 512, 512]));
    raw.set(t::WHITE_LEVEL, Value::Short(vec![16_383]));
    raw.set(SONY_WB_RGGB, Value::Short(vec![2048, 1024, 1024, 2048]));
    raw.set_image(ImageData::Strips { rows_per_strip: height, strips: vec![strip] });

    let mut ifd0 = IfdBuilder::new();
    ifd0.set(t::MAKE, Value::Ascii("SONY".into()));
    ifd0.set(t::MODEL, Value::Ascii("ILCE-7M4".into()));
    ifd0.set(t::ORIENTATION, Value::Short(vec![1]));
    ifd0.add_sub_ifd(raw);
    TiffWriter::default().write(&[ifd0]).map_err(|e| format!("write synthetic ARW: {e}"))
}

/// Build deterministic opaque RGBA PNG pixels with an sRGB chunk and no external assets.
pub fn procedural_png(width: u32, height: u32) -> Result<Vec<u8>, String> {
    validate_dimensions(width, height)?;
    let capacity = width.checked_mul(height).and_then(|px| px.checked_mul(4)).ok_or_else(|| "PNG dimensions overflow".to_string())? as usize;
    let mut pixels = Vec::with_capacity(capacity);
    for y in 0..height {
        for x in 0..width {
            let checker = ((x / 12) + (y / 12)) % 2;
            let r = ((x * 255) / width.max(1)) as u8;
            let g = ((y * 255) / height.max(1)) as u8;
            let b = if checker == 0 { 56 } else { 212 };
            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }
    let mut encoded = Vec::new();
    let mut encoder = png::Encoder::new(&mut encoded, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    let mut writer = encoder.write_header().map_err(|e| format!("PNG header: {e}"))?;
    writer.write_image_data(&pixels).map_err(|e| format!("PNG pixels: {e}"))?;
    drop(writer);
    Ok(encoded)
}

/// Build a minimal native SQLite database containing two Adobe images (master + virtual copy),
/// one nested collection, and membership rows. Paths are absolute so import exercises relinking.
pub fn synthetic_lightroom_catalog(root: &Path, file_name: &str) -> Result<Vec<u8>, String> {
    if file_name.is_empty() || Path::new(file_name).components().count() != 1 {
        return Err("catalog file name must be a single non-empty path component".into());
    }
    let absolute_root = std::fs::canonicalize(root).map_err(|e| format!("canonicalize catalog root: {e}"))?;
    let root_text = absolute_root.to_string_lossy().into_owned();
    let tables = vec![
        Table::new("AgLibraryRootFolder", "id_local INTEGER PRIMARY KEY, absolutePath", vec![N, T(root_text)]),
        Table::new("AgLibraryFolder", "id_local INTEGER PRIMARY KEY, rootFolder, pathFromRoot", vec![N, I(1), T("Photos/".into())]),
        Table::new("AgLibraryFile", "id_local INTEGER PRIMARY KEY, folder, idx_filename", vec![N, I(1), T(file_name.into())]),
        Table::rows(
            "Adobe_images",
            "id_local INTEGER PRIMARY KEY, id_global, rootFile, fileFormat, fileWidth, fileHeight, rating, pick, masterImage, copyName, captureTime",
            vec![
                vec![
                    N,
                    T("synthetic-master".into()),
                    I(1),
                    T("ARW".into()),
                    I(ARW_WIDTH as i64),
                    I(ARW_HEIGHT as i64),
                    I(5),
                    I(1),
                    I(0),
                    T(String::new()),
                    T("2026-10-08T12:00:00".into()),
                ],
                vec![
                    N,
                    T("synthetic-copy".into()),
                    I(1),
                    T("ARW".into()),
                    I(ARW_WIDTH as i64),
                    I(ARW_HEIGHT as i64),
                    I(3),
                    I(-1),
                    I(1),
                    T("Virtual Mono".into()),
                    T("2026-10-08T12:00:00".into()),
                ],
            ],
        ),
        Table::rows(
            "AgLibraryCollection",
            "id_local INTEGER PRIMARY KEY, parent, name, creationId",
            vec![
                vec![N, I(0), T("Synthetic Lightroom".into()), T("collection.group".into())],
                vec![N, I(1), T("Keepers".into()), T("collection".into())],
            ],
        ),
        Table::rows(
            "AgLibraryCollectionImage",
            "collection, image, positionInCollection",
            vec![vec![I(2), I(2), T("a".into())], vec![I(2), I(1), T("b".into())]],
        ),
    ];
    sqlite_fixture(&tables)
}

fn validate_dimensions(width: u32, height: u32) -> Result<(), String> {
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(format!("dimensions outside 1..={MAX_DIMENSION}: {width}x{height}"));
    }
    Ok(())
}

#[derive(Clone, Debug)]
enum SqlValue {
    Null,
    Integer(i64),
    Text(String),
}
use SqlValue::{Integer as I, Null as N, Text as T};

#[derive(Clone, Debug)]
struct Table {
    name: &'static str,
    columns: &'static str,
    rows: Vec<Vec<SqlValue>>,
}

impl Table {
    fn new(name: &'static str, columns: &'static str, row: Vec<SqlValue>) -> Self {
        Self { name, columns, rows: vec![row] }
    }

    fn rows(name: &'static str, columns: &'static str, rows: Vec<Vec<SqlValue>>) -> Self {
        Self { name, columns, rows }
    }
}

fn sqlite_fixture(tables: &[Table]) -> Result<Vec<u8>, String> {
    let page_size = 2048usize;
    let page_count = 1usize.checked_add(tables.len()).ok_or_else(|| "SQLite page count overflow".to_string())?;
    let mut bytes = vec![0u8; page_count.saturating_mul(page_size)];
    bytes[..16].copy_from_slice(b"SQLite format 3\0");
    bytes[16..18].copy_from_slice(&(page_size as u16).to_be_bytes());
    bytes[18] = 1;
    bytes[19] = 1;
    bytes[21] = 64;
    bytes[22] = 32;
    bytes[23] = 32;
    bytes[28..32].copy_from_slice(&(page_count as u32).to_be_bytes());
    // Use schema format 4, matching SQLite's current format and engine fixture parser.
    bytes[44..48].copy_from_slice(&4u32.to_be_bytes());
    bytes[56..60].copy_from_slice(&1u32.to_be_bytes());

    let mut schema = Vec::with_capacity(tables.len());
    for (index, table) in tables.iter().enumerate() {
        if table.rows.is_empty() {
            return Err(format!("SQLite table {} has no row", table.name));
        }
        schema.push(vec![
            T("table".into()),
            T(table.name.into()),
            T(table.name.into()),
            I(index as i64 + 2),
            T(format!("CREATE TABLE {} ({})", table.name, table.columns)),
        ]);
        let page_start = (index + 1) * page_size;
        sqlite_leaf(&mut bytes[page_start..page_start + page_size], 0, &table.rows)?;
    }
    sqlite_leaf(&mut bytes[..page_size], 100, &schema)?;
    Ok(bytes)
}

fn sqlite_varint(mut value: u64) -> Vec<u8> {
    let mut bytes = vec![(value & 0x7f) as u8];
    value >>= 7;
    while value != 0 {
        bytes.push(((value & 0x7f) as u8) | 0x80);
        value >>= 7;
    }
    bytes.reverse();
    bytes
}

fn sqlite_record(values: &[SqlValue]) -> Vec<u8> {
    let mut header = Vec::new();
    let mut body = Vec::new();
    for value in values {
        let serial = match value {
            SqlValue::Null => 0,
            SqlValue::Integer(number) => {
                body.extend_from_slice(&number.to_be_bytes());
                6
            }
            SqlValue::Text(text) => {
                body.extend_from_slice(text.as_bytes());
                13 + (text.len() as u64 * 2)
            }
        };
        header.extend(sqlite_varint(serial));
    }
    let mut record = sqlite_varint((header.len() + 1) as u64);
    record.extend(header);
    record.extend(body);
    record
}

fn sqlite_leaf(page: &mut [u8], offset: usize, rows: &[Vec<SqlValue>]) -> Result<(), String> {
    if page.len() < offset.saturating_add(8) || rows.len() > usize::from(u16::MAX) {
        return Err("SQLite page cannot hold rows".into());
    }
    page[offset] = 13;
    page[offset + 3..offset + 5].copy_from_slice(&(rows.len() as u16).to_be_bytes());
    let mut end = page.len();
    for (index, values) in rows.iter().enumerate() {
        let payload = sqlite_record(values);
        let mut cell = sqlite_varint(payload.len() as u64);
        cell.extend(sqlite_varint(index as u64 + 1));
        cell.extend(payload);
        if end < offset + 8 + (index + 1) * 2 + cell.len() {
            return Err("SQLite page overflow".into());
        }
        end -= cell.len();
        page[end..end + cell.len()].copy_from_slice(&cell);
        let pointer = u16::try_from(end).map_err(|_| "SQLite cell offset overflow")?;
        page[offset + 8 + index * 2..offset + 10 + index * 2].copy_from_slice(&pointer.to_be_bytes());
    }
    let content_start = u16::try_from(end).map_err(|_| "SQLite content offset overflow")?;
    page[offset + 5..offset + 7].copy_from_slice(&content_start.to_be_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_inputs_have_real_signatures_and_bounded_dimensions() -> Result<(), String> {
        let arw = synthetic_sony_arw(32, 16)?;
        assert_eq!(&arw[..4], b"II*\0");
        let png = procedural_png(16, 8)?;
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert!(procedural_png(MAX_DIMENSION + 1, 1).is_err());
        Ok(())
    }

    #[test]
    fn catalog_is_real_sqlite_with_nested_collection_rows() -> Result<(), String> {
        let catalog = synthetic_lightroom_catalog(Path::new("."), "synthetic-sonya-01.arw")?;
        assert_eq!(&catalog[..16], b"SQLite format 3\0");
        assert_eq!(u16::from_be_bytes([catalog[16], catalog[17]]), 2048);
        assert_eq!(u32::from_be_bytes([catalog[44], catalog[45], catalog[46], catalog[47]]), 4);
        assert!(catalog.windows(b"Synthetic Lightroom".len()).any(|window| window == b"Synthetic Lightroom"));
        Ok(())
    }
}
