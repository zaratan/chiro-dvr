use super::{href, text};

#[test]
fn a_video_name_cannot_open_a_tag_or_close_an_attribute() {
    assert_eq!(
        text(r#"</script><b a="1">'x' & y"#),
        "&lt;/script&gt;&lt;b a=&quot;1&quot;&gt;&#39;x&#39; &amp; y"
    );
}

#[test]
fn a_path_keeps_its_slashes_and_encodes_spaces_accents_and_url_signs() {
    assert_eq!(
        href("nuit 12 #1 été", "extraits/piste_01_0m04s.mp4"),
        "nuit%2012%20%231%20%C3%A9t%C3%A9/extraits/piste_01_0m04s.mp4"
    );
}

#[test]
fn a_percent_sign_or_a_question_mark_in_a_name_stays_part_of_the_file() {
    assert_eq!(href("v", "100%?.png"), "v/100%25%3F.png");
}
