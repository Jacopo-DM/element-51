//! Saying a PNG's resolution.

use element_51_core::png::{crc32, with_dpi};

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

/// A PNG made of the signature, an IHDR chunk and an IEND chunk (enough for
/// the header logic; no program needs to decode it here).
fn tiny_png() -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&SIGNATURE);
    let mut header = b"IHDR".to_vec();
    header.extend_from_slice(&[0, 0, 0, 1, 0, 0, 0, 1, 8, 0, 0, 0, 0]);
    data.extend_from_slice(&13u32.to_be_bytes());
    data.extend_from_slice(&header);
    data.extend_from_slice(&crc32(&header).to_be_bytes());
    let end = b"IEND".to_vec();
    data.extend_from_slice(&0u32.to_be_bytes());
    data.extend_from_slice(&end);
    data.extend_from_slice(&crc32(&end).to_be_bytes());
    data
}

#[test]
fn the_crc_matches_the_standard_test_value() {
    assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
}

#[test]
fn a_chunk_for_300_dpi_is_added_after_the_header() {
    let original = tiny_png();
    let made = with_dpi(&original, 300).unwrap();
    assert_eq!(made.len(), original.len() + 21);
    assert_eq!(&made[..33], &original[..33]);
    assert_eq!(&made[33..37], &9u32.to_be_bytes());
    assert_eq!(&made[37..41], b"pHYs");
    assert_eq!(&made[41..45], &11811u32.to_be_bytes());
    assert_eq!(&made[45..49], &11811u32.to_be_bytes());
    assert_eq!(made[49], 1);
    assert_eq!(&made[50..54], &crc32(&made[37..50]).to_be_bytes());
    assert_eq!(&made[54..], &original[33..]);
}

#[test]
fn something_that_is_not_a_png_is_refused() {
    assert_eq!(with_dpi(b"not a png at all, really not one", 300), None);
    assert_eq!(with_dpi(&SIGNATURE, 300), None);
    let mut wrong = tiny_png();
    wrong[12] = b'X';
    assert_eq!(with_dpi(&wrong, 300), None);
}
