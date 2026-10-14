use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use draw::canvas::{Canvas, PixelBox};
use draw::color::{PALETTE, Rgb};
use png::{BitDepth, ColorType, Decoder, EncodingError};
use tempfile::TempDir;

use super::{Error, encoding_error, write_png};

fn painted() -> Canvas {
    let mut canvas = Canvas::new(7, 5, Rgb::new(1, 2, 3)).unwrap();
    let corner = PixelBox {
        left: 2,
        top: 1,
        right: 4,
        bottom: 3,
    };
    canvas.fill_rectangle(corner, PALETTE[4]);
    canvas
}

fn read_back(path: &Path) -> (u32, u32, ColorType, BitDepth, Vec<u8>) {
    let mut reader = Decoder::new(BufReader::new(File::open(path).unwrap()))
        .read_info()
        .unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut pixels).unwrap();
    pixels.truncate(info.buffer_size());
    (info.width, info.height, info.color_type, info.bit_depth, pixels)
}

#[test]
fn a_written_summary_reads_back_as_the_same_rgb_pixels_without_alpha() {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("video.png");
    let canvas = painted();

    write_png(&path, &canvas).unwrap();

    let (width, height, color, depth, pixels) = read_back(&path);
    assert_eq!((width, height, color, depth), (7, 5, ColorType::Rgb, BitDepth::Eight));
    let expected: Vec<u8> = canvas
        .rgba()
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|pixel| pixel[..3].to_vec())
        .collect();
    assert_eq!(pixels, expected);
}

#[test]
fn an_earlier_summary_is_replaced_and_no_temporary_file_is_left() {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("video.png");
    std::fs::write(&path, b"old").unwrap();

    write_png(&path, &painted()).unwrap();

    assert_eq!(read_back(&path).4.len(), 7 * 5 * 3);
    let names: Vec<_> = std::fs::read_dir(folder.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names, ["video.png"]);
}

#[test]
fn a_missing_folder_is_a_typed_error_naming_the_image() {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("missing").join("video.png");

    let failed = write_png(&path, &painted()).unwrap_err();

    assert!(
        matches!(&failed, Error::Create { path: named, .. } if *named == path),
        "{failed:?}"
    );
    assert!(!path.exists());
}

#[cfg(unix)]
#[test]
fn a_summary_gets_the_same_permissions_as_any_file_the_user_creates() {
    use std::os::unix::fs::PermissionsExt;
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("video.png");
    let ordinary = folder.path().join("ordinary");
    std::fs::write(&ordinary, b"").unwrap();

    write_png(&path, &painted()).unwrap();

    let mode = |path: &Path| std::fs::metadata(path).unwrap().permissions().mode();
    assert_eq!(mode(&path), mode(&ordinary));
}

#[test]
fn a_disk_error_while_encoding_comes_out_as_a_write_error_with_its_cause() {
    let full = EncodingError::IoError(std::io::Error::from(std::io::ErrorKind::StorageFull));
    let error = encoding_error(Path::new("out/v_resume.png"), full);
    assert!(
        matches!(&error, Error::Write { source, .. } if source.kind() == std::io::ErrorKind::StorageFull),
        "{error:?}"
    );
}
