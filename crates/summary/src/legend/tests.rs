use std::num::NonZeroU32;

use draw::canvas::Canvas;
use draw::color::{PALETTE, PANEL, Rgb};
use draw::style::{SummaryStyle, style_for};
use draw::text::Typeface;

use super::{ELLIPSIS, LegendEntry, column_width, fit_text, legend_panel, legend_width, row_height};

fn tools() -> (Typeface, SummaryStyle) {
    let typeface = Typeface::roboto().unwrap();
    let style = style_for(NonZeroU32::new(1440).unwrap(), "99", &typeface);
    (typeface, style)
}

fn entries(count: usize) -> Vec<LegendEntry> {
    (1..=count)
        .map(|id| LegendEntry {
            label: id.to_string(),
            color: PALETTE[0],
            text: "0:07".to_owned(),
        })
        .collect()
}

fn pixels(canvas: &Canvas) -> Vec<Rgb> {
    canvas
        .rgba()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|pixel| Rgb::new(pixel[0], pixel[1], pixel[2]))
        .collect()
}

#[test]
fn panel_has_the_video_height_and_one_column_when_everything_fits() {
    let (typeface, style) = tools();
    let panel = legend_panel(&typeface, &["video".to_owned()], &entries(3), 1080, &style).unwrap();
    let expected = (column_width(&typeface, &style, &[]) + style.margin).ceil() as u32;
    assert_eq!((panel.width(), panel.height()), (expected, 1080));
}

#[test]
fn panel_adds_columns_instead_of_dropping_tracks() {
    let (typeface, style) = tools();
    let rows_in_one_column = (1080.0 / row_height(&style)).floor() as usize;
    let panel = legend_panel(
        &typeface,
        &["video".to_owned()],
        &entries(2 * rows_in_one_column),
        1080,
        &style,
    )
    .unwrap();
    assert!(panel.width() as f32 >= 2.0 * column_width(&typeface, &style, &[]));
}

#[test]
fn each_entry_shows_its_track_color() {
    let (typeface, style) = tools();
    let panel = legend_panel(&typeface, &["video".to_owned()], &entries(1), 1080, &style).unwrap();
    let all = pixels(&panel);
    assert!(all.contains(&PALETTE[0]));
    assert_eq!(all.last(), Some(&PANEL));
}

#[test]
fn long_video_name_is_cut_with_an_ellipsis() {
    let (typeface, style) = tools();
    let name = "a_very_long_video_name_from_another_camera";
    let text = fit_text(&typeface, name, 150.0, &style);
    assert!(text.ends_with(ELLIPSIS));
    assert!(text.len() < name.len());
    assert!(typeface.measure(&text, style.text) <= 150.0);
}

#[test]
fn short_text_is_left_untouched() {
    let (typeface, style) = tools();
    assert_eq!(fit_text(&typeface, "092", 500.0, &style), "092");
}

#[test]
fn column_widens_for_start_times_past_one_hundred_minutes() {
    let (typeface, style) = tools();
    assert!(column_width(&typeface, &style, &["100:00"]) > column_width(&typeface, &style, &["0:07"]));
}

#[test]
fn a_long_header_widens_the_panel_up_to_three_columns_and_no_further() {
    let (typeface, style) = tools();
    let column = column_width(&typeface, &style, &[]);
    let medium = "video_092_original_3m24".to_owned();
    let medium_width = typeface.measure(&medium, style.text) + 2.0 * style.margin;
    let panel = legend_panel(&typeface, &[medium], &entries(1), 1080, &style).unwrap();
    assert!(medium_width > column && medium_width < 3.0 * column);
    assert_eq!(panel.width(), medium_width.ceil() as u32);
    let huge = "a".repeat(400);
    let capped = legend_panel(&typeface, &[huge], &entries(1), 1080, &style).unwrap();
    assert_eq!(capped.width(), (3.0 * column).ceil() as u32);
}

#[test]
fn the_widest_header_line_is_never_cut_by_the_rounding_of_its_own_width() {
    let (typeface, style) = tools();
    let header = ["video_121", "2 pistes", "0:00 - 5:00", "illisible 0:04 - 0:05"].map(String::from);
    let column = column_width(&typeface, &style, &[]);
    let width = legend_width(&typeface, &header, column + style.margin, column, &style);
    for line in &header {
        assert_eq!(
            &fit_text(&typeface, line, width as f32 - 2.0 * style.margin, &style),
            line
        );
    }
}

#[test]
fn only_the_video_name_is_capped_at_three_columns_so_the_unreadable_total_is_never_cut() {
    let (typeface, style) = tools();
    let total = "+ 84 autres, 10 min 0 s illisibles en tout".to_owned();
    let header = ["a".repeat(400), total.clone()];
    let column = column_width(&typeface, &style, &[]);
    let width = legend_width(&typeface, &header, column + style.margin, column, &style);
    assert!(typeface.measure(&total, style.text) > 3.0 * column - 2.0 * style.margin);
    assert_eq!(
        fit_text(&typeface, &total, width as f32 - 2.0 * style.margin, &style),
        total
    );
    assert!(fit_text(&typeface, &header[0], width as f32 - 2.0 * style.margin, &style).ends_with(ELLIPSIS));
}
