use std::error::Error as _;
use std::str::FromStr;

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{Arg, Command};
use engine::settings::{Bound, Region};

use super::{Expected, Reader, Unreadable};

const POSITIVE: Expected = Expected::Setting(Bound::Positive);

fn read<T>(reader: Reader<T>, value: &str) -> Result<T, clap::Error>
where
    T: FromStr + Clone + Send + Sync + 'static,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let command = Command::new("batdetect-rs").arg(Arg::new("threshold").long("threshold").value_parser(reader));
    command
        .try_get_matches_from(["batdetect-rs", "--threshold", value])
        .map(|mut matches| matches.remove_one::<T>("threshold").unwrap())
}

fn refused<T>(reader: Reader<T>, value: &str) -> (String, Expected, String, Option<String>, String)
where
    T: FromStr + Clone + Send + Sync + 'static,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let error = read(reader, value).err().unwrap();
    assert_eq!(error.kind(), ErrorKind::ValueValidation);
    let unreadable = error.source().unwrap().downcast_ref::<Unreadable>().unwrap();
    (
        unreadable.option.clone(),
        unreadable.expected,
        unreadable.written.clone(),
        unreadable.pointed.clone(),
        unreadable.source().unwrap().to_string(),
    )
}

#[test]
fn a_value_that_reads_is_kept() {
    assert_eq!(read(Reader::<f64>::decimal(POSITIVE), "11.5").unwrap(), 11.5);
    assert_eq!(
        read(Reader::<Region>::exact(Expected::Region), "0,0,1,0.05").unwrap(),
        Region {
            x0: 0.0,
            y0: 0.0,
            x1: 1.0,
            y1: 0.05
        }
    );
}

#[test]
fn a_refused_value_keeps_its_option_its_range_the_value_as_written_and_the_reason_of_the_library() {
    assert_eq!(
        refused(Reader::<usize>::exact(Expected::Setting(Bound::AtLeast(0))), "many"),
        (
            "--threshold".to_owned(),
            Expected::Setting(Bound::AtLeast(0)),
            "many".to_owned(),
            None,
            "invalid digit found in string".to_owned()
        )
    );
}

#[test]
fn a_decimal_comma_is_answered_with_the_same_number_written_with_a_point() {
    assert_eq!(
        refused(Reader::<f64>::decimal(POSITIVE), "11,5").3.as_deref(),
        Some("11.5")
    );
}

#[test]
fn an_integer_option_or_a_comma_that_still_does_not_read_gets_no_advice_about_the_point() {
    assert_eq!(
        refused(Reader::<usize>::exact(Expected::Setting(Bound::AtLeast(0))), "1,5").3,
        None
    );
    assert_eq!(refused(Reader::<f64>::decimal(POSITIVE), "a,b").3, None);
}

#[test]
fn an_empty_value_is_told_as_a_missing_value_and_not_as_an_unreadable_one() {
    let error = read(Reader::<f64>::decimal(POSITIVE), "").err().unwrap();
    assert_eq!(error.kind(), ErrorKind::InvalidValue);
    assert!(matches!(error.get(ContextKind::InvalidValue), Some(ContextValue::String(value)) if value.is_empty()));
    assert!(error.source().is_none());
}
