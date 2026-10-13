use draw::canvas::PixelBox;
use media::convert::Region;

#[cfg(test)]
mod tests;

const ZONE_STEP: u32 = 16;

pub(crate) fn merged(mut zones: Vec<Region>, width: u32, height: u32) -> Vec<Region> {
    while let Some((first, second)) = touching(&zones) {
        let other = zones.swap_remove(second);
        if let Some(kept) = zones.get_mut(first) {
            *kept = round_region(hull(*kept, other), width, height);
        }
    }
    zones
}

pub(crate) fn even_region(extent: PixelBox, width: u32, height: u32) -> Option<Region> {
    let start = |value: i32| u32::try_from(value.max(0)).unwrap_or(0) & !1;
    let end = |value: i32, limit: u32| u32::try_from(value.max(0)).unwrap_or(0).saturating_add(2).min(limit) & !1;
    let (left, right) = rounded(start(extent.left), end(extent.right, width), width & !1)?;
    let (top, bottom) = rounded(start(extent.top), end(extent.bottom, height), height & !1)?;
    Some(Region {
        left,
        top,
        width: right - left,
        height: bottom - top,
    })
}

fn round_region(region: Region, width: u32, height: u32) -> Region {
    let across = rounded(region.left, region.left.saturating_add(region.width), width & !1);
    let down = rounded(region.top, region.top.saturating_add(region.height), height & !1);
    match across.zip(down) {
        Some(((left, right), (top, bottom))) => Region {
            left,
            top,
            width: right - left,
            height: bottom - top,
        },
        None => region,
    }
}

fn rounded(start: u32, end: u32, limit: u32) -> Option<(u32, u32)> {
    let length = end.checked_sub(start).filter(|length| *length > 0)?;
    let length = length.div_ceil(ZONE_STEP).saturating_mul(ZONE_STEP).min(limit);
    let start = start.min(limit - length);
    Some((start, start + length))
}

fn touching(zones: &[Region]) -> Option<(usize, usize)> {
    let meet = |a: &Region, b: &Region| {
        a.left <= b.left + b.width
            && b.left <= a.left + a.width
            && a.top <= b.top + b.height
            && b.top <= a.top + a.height
    };
    zones.iter().enumerate().find_map(|(first, zone)| {
        zones
            .iter()
            .enumerate()
            .skip(first + 1)
            .find(|(_, other)| meet(zone, other))
            .map(|(second, _)| (first, second))
    })
}

fn hull(a: Region, b: Region) -> Region {
    let left = a.left.min(b.left);
    let top = a.top.min(b.top);
    let right = (a.left + a.width).max(b.left + b.width);
    let bottom = (a.top + a.height).max(b.top + b.height);
    Region {
        left,
        top,
        width: right - left,
        height: bottom - top,
    }
}
