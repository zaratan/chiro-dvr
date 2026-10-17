use super::EdgeLosses;

#[test]
fn the_union_of_two_readings_keeps_the_larger_loss_at_each_edge() {
    let before = EdgeLosses {
        before_first: 3,
        after_last: 0,
    };
    let after = EdgeLosses {
        before_first: 1,
        after_last: 2,
    };

    assert_eq!(
        before.union(after),
        EdgeLosses {
            before_first: 3,
            after_last: 2
        }
    );
    assert_eq!(
        EdgeLosses::default().union(EdgeLosses::default()),
        EdgeLosses::default()
    );
}

#[test]
fn edge_losses_are_written_with_the_names_of_the_reference() {
    let losses = EdgeLosses {
        before_first: 2,
        after_last: 0,
    };

    let written = serde_json::to_value(losses).unwrap();

    assert_eq!(written, serde_json::json!({"before_first": 2, "after_last": 0}));
    assert_eq!(serde_json::from_value::<EdgeLosses>(written).unwrap(), losses);
}
