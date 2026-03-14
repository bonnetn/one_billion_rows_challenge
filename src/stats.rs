#![allow(dead_code)]

use std::fmt::{Display, Formatter};

use anyhow::Context as _;


#[inline]
pub fn average(sum: i32, count: usize) -> Result<i16, anyhow::Error> {
    let count = count as i32;
    let avg = sum / count;
    let mut avg: i16 = avg.try_into().context("Average is too large")?;
    let avg_rest = sum % count;

    if 2 * avg_rest >= count {
        avg += 1;
    }

    if 2 * avg_rest < -count {
        avg -= 1;
    }

    Ok(avg)
}

pub struct Value(pub i16);

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let raw = self.0;
        let sign = if raw < 0 { "-" } else { "" };
        let abs = raw.abs();
        let int = abs / 10;
        let frac = abs % 10;

        write!(f, "{}{}.{}", sign, int, frac)
    }
}