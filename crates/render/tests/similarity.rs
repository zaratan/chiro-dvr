#[path = "support/similarity.rs"]
mod similarity;

use crate::similarity::{Area, BLOCK, masked_similarity};

#[test]
fn the_similarity_is_one_for_the_same_image_and_known_for_an_offset() {
    let width = 16;
    let first: Vec<u8> = (0..width * BLOCK).map(|index| (index * 7 % 200) as u8).collect();
    let same = masked_similarity(&first, &first, width, BLOCK, &[]);
    assert_eq!((same.mean(), same.blocks), (1.0, 2));
    let flat = vec![100; width * BLOCK];
    let raised = vec![110; width * BLOCK];
    let offset = masked_similarity(&flat, &raised, width, BLOCK, &[]);
    assert!((offset.mean() - 0.995_476_444_091_506_6).abs() < 1e-12, "{offset:?}");
    let hidden = Area {
        left: 9.0,
        top: 0.0,
        right: 10.0,
        bottom: 1.0,
    };
    let masked = masked_similarity(&flat, &raised, width, BLOCK, &[hidden]);
    assert_eq!(masked.blocks, 1);
}
