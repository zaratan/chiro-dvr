use super::Setting;

#[test]
fn a_setting_is_named_by_its_params_json_key() {
    let names: Vec<String> = [
        Setting::PadS,
        Setting::MaxJump,
        Setting::MinTravel,
        Setting::TwinDistance,
        Setting::MaxMedianTurn,
        Setting::RejoinHidden,
        Setting::RejoinDistance,
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    assert_eq!(
        names,
        [
            "pad_s",
            "max_jump",
            "min_travel",
            "twin_distance",
            "max_median_turn",
            "rejoin_hidden",
            "rejoin_distance"
        ]
    );
}
