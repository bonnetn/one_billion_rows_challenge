#![allow(dead_code)]

use anyhow::Result;
use anyhow::{Context as _};
use memchr::memchr;
use memmap2::Mmap;
use rustc_hash::FxHashMap;
use std::io::Write as _;
use std::{collections::BTreeMap, fs::File, path::Path};

use crate::stats::{DecimalValue, average};

const BUFFER_SIZE: usize = 1024 * 1024 * 256;
const STATS_CAPACITY: usize = 10000;

pub fn run(path: &Path) -> Result<String> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let mut slice = &mmap[..];

    let mut line_processor = LineProcessor::new();

    while slice.len() > 0 {
        slice = line_processor.process(slice)?;
    }

    let LineProcessor {
        stats,
        buffer,
        count,
    } = line_processor;
    assert!(buffer.len() == 0);
    println!("Processed {} lines", count);

    let stats = stats
        .into_iter()
        .map(|(name, stats)| {
            let name = String::from_utf8(name).context("Invalid station name")?;
            Ok((name, stats))
        })
        .collect::<Result<BTreeMap<String, Stats>>>()?;

    let mut output = Vec::new();
    write!(output, "{{")?;
    let mut first = true;
    for (name, stats) in stats {
        let min = stats.min;
        let max = stats.max;
        let avg = average(stats.sum, stats.count);
        if !first {
            write!(output, ", ")?;
        }
        write!(
            output,
            "{}={}/{}/{}",
            name,
            DecimalValue(min),
            DecimalValue(avg),
            DecimalValue(max)
        )?;
        first = false;
    }
    writeln!(output, "}}")?;

    Ok(String::from_utf8(output)?)
}

struct Stats {
    min: i16,
    max: i16,
    sum: i32,
    count: usize,
}

struct LineProcessor {
    stats: FxHashMap<Vec<u8>, Stats>,
    buffer: Vec<u8>,
    count: usize,
}

impl LineProcessor {
    fn new() -> Self {
        Self {
            stats: FxHashMap::with_capacity_and_hasher(STATS_CAPACITY, Default::default()),
            buffer: Vec::with_capacity(128),
            count: 0,
        }
    }

    fn process<'a>(&mut self, input_data: &'a [u8]) -> Result<&'a [u8]> {
        if self.buffer.len() > 0 {
            // We already have a partial line in memory, we need to stich it with the incoming line.
            let idx = input_data.iter().position(|&b| b == b'\n');
            let Some(idx) = idx else {
                // The buffered data + incoming line STILL doesn't form a full line, we save the partial line for next time.
                self.buffer.extend_from_slice(input_data);
                return Ok(&[]);
            };

            println!("Stiching, size = {}", self.buffer.len());
            let line = &input_data[..idx + 1];
            self.buffer.extend_from_slice(line);
            let Some((station_name, value, result)) = _process(&self.buffer)? else {
                panic!(
                    "We should have fed a full line, but for some reason we could not parse the full thing"
                );
            };
            assert!(result.len() == 0);
            self.count += 1;
            update_stats(&mut self.stats, station_name, value);
            self.buffer.clear();

            return Ok(&input_data[idx + 1..]);
        }

        let Some((station_name, value, rest)) = _process(input_data)? else {
            // The line is not full, we save the partial line for next time.
            self.buffer.extend_from_slice(input_data);
            return Ok(&[]);
        };

        self.count += 1;
        if self.count % 1_000_000 == 0 {
            println!("Processed {}M lines", self.count / 1_000_000);
        }

        update_stats(&mut self.stats, station_name, value);

        return Ok(rest);
    }
}

fn update_stats(stats: &mut FxHashMap<Vec<u8>, Stats>, station_name: &[u8], value: i16) {
    if let Some(stats) = stats.get_mut(station_name) {
        stats.min = stats.min.min(value);
        stats.max = stats.max.max(value);
        stats.sum += value as i32;
        stats.count += 1;
    } else {
        stats.insert(
            station_name.to_vec(),
            Stats {
                min: value,
                max: value,
                sum: value as i32,
                count: 1,
            },
        );
    }
}
fn _process<'a>(input_data: &'a [u8]) -> Result<Option<(&'a [u8], i16, &'a [u8])>> {
    let rest = input_data;

    let Some((station_name, rest)) = split_at_char(rest, b';')? else {
        return Ok(None);
    };

    let Some((value, rest)) = parse_value(rest)? else {
        return Ok(None);
    };

    Ok(Some((station_name, value, rest)))
}

fn split_at_char(input_data: &[u8], char: u8) -> Result<Option<(&[u8], &[u8])>> {
    let idx = memchr(char, input_data);
    if let Some(idx) = idx {
        Ok(Some((&input_data[..idx], &input_data[idx + 1..])))
    } else {
        Ok(None)
    }
}

fn parse_value<'a>(mut d: &'a [u8]) -> Result<Option<(i16, &'a [u8])>> {
    let mut negative = false;
    let mut result = 0_i16;

    while let [char, rest @ ..] = d {
        d = rest;
        match char {
            b'-' => {
                negative = true;
            }
            b'.' => {}
            b'\n' => {
                if negative {
                    result = -result;
                }
                return Ok(Some((result, rest)));
            }
            c => {
                assert!(b'0' <= *c && *c <= b'9', "Invalid digit: {}", *c as char);
                result = result * 10 + (*c - b'0') as i16;
            }
        }
    }

    Ok(None)
}
