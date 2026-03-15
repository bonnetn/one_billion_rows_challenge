use anyhow::Result;
use anyhow::{Context as _, bail};
use memmap2::Mmap;
use std::io::Write as _;
use std::thread;
use std::{fs::File, path::Path};

use crate::stats::StationStatsMap;
use crate::{station_name, value};

fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

pub fn run(path: &Path) -> Result<String> {
    let file = File::open(path)?;
    // SAFETY: mmap is only used for reading and we do not modify the file.
    let mmap = unsafe { Mmap::map(&file)? };
    mmap.advise(memmap2::Advice::Sequential)?;

    let count = thread::available_parallelism()?.get();
    let chunk_size = mmap.len().div_ceil(count);

    println!("File size: {}", mmap.len());
    println!("Available parallelism: {count}");
    println!("Chunk size: {chunk_size}");

    let total_stats = thread::scope(|s| {
        let mut handles = Vec::new();

        let mut previous_end = 0_usize;
        for i in 0..count {
            let start = previous_end;
            let tentative_end = (start + chunk_size).min(mmap.len());
            let end = if let Some(idx) = memchr(b'\n', &mmap[tentative_end..]) {
                // Include the newline so the next chunk starts at a line boundary
                tentative_end + idx + 1
            } else {
                mmap.len()
            };
            previous_end = end;

            let chunk = &mmap[start..end];
            println!("[{i:02}] Chunk size: {}", chunk.len());

            let t = s.spawn(move || {
                let stats = process(chunk).context("Error processing chunk")?;
                Ok::<_, anyhow::Error>(stats)
            });
            handles.push(t);
        }

        if previous_end != mmap.len() {
            bail!("Chunk boundary error: did not cover full file");
        }

        let mut total_stats = StationStatsMap::new();
        for handle in handles {
            let stats = handle
                .join()
                .expect("worker thread panicked")
                .context("Error joining thread")?;
            total_stats.extend(&stats);
        }

        Ok::<_, anyhow::Error>(total_stats)
    })
    .context("Error processing chunks")?;

    let mut output = Vec::new();
    let report = total_stats.report()?;
    write!(output, "{report}")?;
    Ok(String::from_utf8(output)?)
}

fn process(input_data: &[u8]) -> Result<StationStatsMap> {
    let mut data = input_data;

    let mut stats = StationStatsMap::new();

    let mut count = 0_i32;

    while !data.is_empty() {
        let Some((station_name, rest)) = station_name::parse(data) else {
            let preview: String = String::from_utf8_lossy(&data[..data.len().min(80)]).into();
            bail!("Invalid line, no semicolon found (preview): {:?}", preview);
        };

        let Some((value, rest)) = value::parse(rest) else {
            let preview: String = String::from_utf8_lossy(&rest[..rest.len().min(80)]).into();
            bail!("Invalid line, no value found (preview): {:?}", preview);
        };

        let rest = strip_newline(rest);

        data = rest;

        stats.update(station_name, value);
        count += 1_i32;
    }
    println!("Processed {count} lines");

    Ok(stats)
}

#[inline]
fn strip_newline(d: &[u8]) -> &[u8] {
    match *d {
        [b'\n', ref rest @ ..] => rest,
        _ => d,
    }
}
