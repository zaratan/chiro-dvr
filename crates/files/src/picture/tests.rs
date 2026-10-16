use std::fs;

use super::png_size;

fn header(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    bytes.extend(width.to_be_bytes());
    bytes.extend(height.to_be_bytes());
    bytes.extend([8, 6, 0, 0, 0]);
    bytes
}

#[test]
fn the_size_of_a_summary_image_is_read_from_its_header() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("v_resume.png");
    fs::write(&path, header(1989, 1215)).unwrap();
    assert_eq!(png_size(&path), Some((1989, 1215)));
}

#[test]
fn a_file_that_is_not_a_png_or_is_cut_short_has_no_size() {
    let folder = tempfile::tempdir().unwrap();
    let text = folder.path().join("notes.png");
    fs::write(&text, "not an image at all, really not").unwrap();
    let cut = folder.path().join("cut.png");
    fs::write(&cut, &header(10, 10)[..20]).unwrap();
    assert_eq!(png_size(&text), None);
    assert_eq!(png_size(&cut), None);
    assert_eq!(png_size(&folder.path().join("absent.png")), None);
}
