use super::encoded;

#[test]
fn bytes_are_encoded_as_the_browser_decodes_them_with_padding() {
    for (raw, base64) in [
        (&b""[..], ""),
        (b"f", "Zg=="),
        (b"fo", "Zm8="),
        (b"foo", "Zm9v"),
        (b"foobar", "Zm9vYmFy"),
    ] {
        assert_eq!(encoded(raw), base64);
    }
}

#[test]
fn any_byte_is_carried_including_latin_one_and_markup() {
    assert_eq!(encoded(b"vid\xe9o</template>\xff"), "dmlk6W88L3RlbXBsYXRlPv8=");
}
