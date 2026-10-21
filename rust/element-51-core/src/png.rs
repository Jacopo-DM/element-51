//! The bytes of a PNG file: saying what resolution it has.

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
const HEADER_END: usize = 8 + 4 + 4 + 13 + 4;
const METRE_PER_INCH: f64 = 0.0254;

/// The CRC-32 a PNG chunk ends with.
pub fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// The same PNG with a `pHYs` chunk that says `dpi` dots per inch in both
/// directions, so a program that places it on a page uses the right size. None
/// if `png` is not a PNG that starts with its header chunk.
pub fn with_dpi(png: &[u8], dpi: u32) -> Option<Vec<u8>> {
    if png.len() < HEADER_END || png[..8] != SIGNATURE || &png[12..16] != b"IHDR" {
        return None;
    }
    let per_metre = (f64::from(dpi) / METRE_PER_INCH).round() as u32;
    let mut data = Vec::with_capacity(9);
    data.extend_from_slice(&per_metre.to_be_bytes());
    data.extend_from_slice(&per_metre.to_be_bytes());
    data.push(1);
    let mut named = b"pHYs".to_vec();
    named.extend_from_slice(&data);
    let mut out = Vec::with_capacity(png.len() + 21);
    out.extend_from_slice(&png[..HEADER_END]);
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(&named);
    out.extend_from_slice(&crc32(&named).to_be_bytes());
    out.extend_from_slice(&png[HEADER_END..]);
    Some(out)
}
