pub fn parse(d: &[u8]) -> Option<(i16, &[u8])> {
    match *d {
        [b'-', ref d0, ref d1, b'.', ref f0, ref rest @ ..] => {
            let result = calculate_value(*d0, *d1, *f0)?;
            Some((-result, rest))
        }
        [b'-', ref d1, b'.', ref f0, ref rest @ ..] => {
            let result = calculate_value(b'0', *d1, *f0)?;
            Some((-result, rest))
        }
        [ref d0, ref d1, b'.', ref f0, ref rest @ ..] => {
            let result = calculate_value(*d0, *d1, *f0)?;
            Some((result, rest))
        }
        [ref d1, b'.', ref f0, ref rest @ ..] => {
            let result = calculate_value(b'0', *d1, *f0)?;
            Some((result, rest))
        }
        _ => None,
    }
}

fn calculate_value(d0: u8, d1: u8, f0: u8) -> Option<i16> {
    let d0 = d0.checked_sub(b'0')?;
    let d1 = d1.checked_sub(b'0')?;
    let f0 = f0.checked_sub(b'0')?;
    if d0 > 9 || d1 > 9 || f0 > 9 {
        return None;
    }
    Some(i16::from(d0) * 100 + i16::from(d1) * 10 + i16::from(f0))
}
