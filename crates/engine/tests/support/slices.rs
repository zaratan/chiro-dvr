use std::collections::BTreeSet;

use media::decode::{Packet, PacketSeam};

const LENGTH: usize = 4;
const NAL_TYPE_MASK: u8 = 0x1f;
const SLICE: u8 = 1;
const IDR_SLICE: u8 = 5;

pub(crate) fn cutting_the_last_third_of(packets: BTreeSet<usize>) -> Box<dyn PacketSeam> {
    Box::new(move |packet: &mut Packet<'_>| {
        if packets.contains(&packet.ordinal()) {
            let kept = cut_slice(packet.data_mut().unwrap());
            packet.shrink(kept);
        }
    })
}

fn cut_slice(data: &mut [u8]) -> usize {
    let mut at = 0;
    while at + LENGTH < data.len() {
        let length = usize::try_from(u32::from_be_bytes(data[at..at + LENGTH].try_into().unwrap())).unwrap();
        let unit = at + LENGTH;
        if matches!(data[unit] & NAL_TYPE_MASK, SLICE | IDR_SLICE) {
            let removed = (length / 3).max(1);
            let shortened = u32::try_from(length - removed).unwrap();
            data[at..unit].copy_from_slice(&shortened.to_be_bytes());
            data.copy_within(unit + length.., unit + length - removed);
            return data.len() - removed;
        }
        at = unit + length;
    }
    panic!("no slice in the packet")
}
