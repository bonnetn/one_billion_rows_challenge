const MAX_LENGTH: usize = 100;

/// Station names are between 1 and 100 characters long.
#[inline]
pub fn parse<'a>(line: &'a [u8]) -> Option<(&'a [u8], &'a [u8])> {
    if line.len() < 2 {
        return None;
    }

    // + 1 for the semicolon
    let end = line.len().min(MAX_LENGTH + 1);

    // NOTE: We could start at 1, but it confuses LLVM and adds bound checks.
    let idx = line[..end].iter().position(|&c| c == b';')?;

    Some((&line[..idx], &line[idx + 1..]))
}
