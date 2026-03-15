#![allow(dead_code)]

use anyhow::Result;
use anyhow::{Context as _, bail};
use memmap2::Mmap;
use std::io::Write as _;
use std::thread;
use std::{fs::File, path::Path};

use crate::{station_name, value};
use crate::stats::StationStatsMap;

fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

pub fn run(path: &Path) -> Result<String> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    mmap.advise(memmap2::Advice::Sequential)?;

    let count = thread::available_parallelism()?.get();
    let chunk_size = mmap.len().div_ceil(count);

    println!("File size: {}", mmap.len());
    println!("Available parallelism: {}", count);
    println!("Chunk size: {}", chunk_size);

    let total_stats = thread::scope(|s| {
        let mut handles = Vec::new();

        let mut previous_end = 0;
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

        assert!(previous_end == mmap.len());

        let mut total_stats = StationStatsMap::new();
        for handle in handles {
            let stats = handle.join().unwrap().context("Error joining thread")?;
            total_stats.extend(&stats);
        }

        Ok::<_, anyhow::Error>(total_stats)
    })
    .context("Error processing chunks")?;

    let mut output = Vec::new();
    let report = total_stats.report()?;
    write!(output, "{}", report)?;
    Ok(String::from_utf8(output)?)
}


fn process<'a>(input_data: &'a [u8]) -> Result<StationStatsMap> {
    let mut data = input_data;

    let mut stats = StationStatsMap::new();

    let mut count = 0;

    while !data.is_empty() {
        let Some((station_name, rest)) = station_name::parse(data) else {
            bail!(
                "Invalid line, no semicolon found: {:?}",
                String::from_utf8_lossy(data)
            );
        };

        let Some((value, rest)) = value::parse(rest) else {
            bail!(
                "Invalid line, no value found: {:?}",
                String::from_utf8_lossy(rest)
            );
        };

        let rest = strip_newline(rest);

        data = rest;

        stats.update(station_name, value);
        count += 1;
    }
    println!("Processed {} lines", count);

    Ok(stats)
}


#[inline]
fn strip_newline(d: &[u8]) -> &[u8] {
    match d {
        [b'\n', rest @ ..] => rest,
        _ => d,
    }
}