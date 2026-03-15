use std::simd::cmp::SimdPartialEq as _;
use std::simd::Simd;


/// Station names are between 1 and 100 characters long.
#[inline]
pub fn parse(line: &[u8]) -> Option<(&[u8], &[u8])> {
    if line.len() < 2 {
        return None;
    }

    let idx = find(line, b';')?;
    Some((&line[..idx], &line[idx + 1..]))
}

const SIMD_SIZE: usize = 64;

fn find(buffer: &[u8], needle: u8) -> Option<usize> {
    let mut index = 0;

    while index + SIMD_SIZE <= buffer.len() {
        let bytes = std::simd::u8x64::from_slice(&buffer[index..index + SIMD_SIZE]);
        let mask = bytes.simd_eq(Simd::splat(needle));
        let bits = mask.to_bitmask();

        if bits != 0 {
            let pos = bits.trailing_zeros() as usize;
            return Some(index + pos);
        }

        index += SIMD_SIZE;
    }

    for i in index..buffer.len() {
        if buffer[i] == needle {
            return Some(i);
        }
    }

    None
}