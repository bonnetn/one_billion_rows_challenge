pub const fn parse<'a>(d: &'a [u8]) -> Option<(i16, &'a [u8])> {
    match d {
        [b'-', d0, d1, b'.', f0, rest @ ..] => {
            let Some(result) = calculate_value(*d0, *d1, *f0) else { return None };
            Some((-result, rest))
        }
        [b'-', d1, b'.', f0, rest @ ..] => {
            let Some(result) = calculate_value(b'0', *d1, *f0) else { return None };
            Some((-result, rest))
        }
        [d0, d1, b'.', f0, rest @ ..] => {
            let Some(result) = calculate_value(*d0, *d1, *f0) else { return None };
            Some((result, rest))
        }
        [d1, b'.', f0, rest @ ..] => {
            let Some(result) = calculate_value(b'0', *d1, *f0) else { return None };
            Some((result, rest))
        }
        _ => None,
    }
}

const fn calculate_value(d0: u8, d1: u8, f0: u8) -> Option<i16> {
    let Some(d0) = d0.checked_sub(b'0') else { return None };
    let Some(d1) = d1.checked_sub(b'0') else { return None };
    let Some(f0) = f0.checked_sub(b'0') else { return None };
    if d0 > 9 || d1 > 9 || f0 > 9 {
        return None;
    }
    Some(d0 as i16 * 100 + d1 as i16 * 10 + f0 as i16)
}