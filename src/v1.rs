use std::fmt::Write;
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
    fs::File,
    io::{BufRead as _, BufReader},
    path::Path,
};

use anyhow::{Context as _, Result};

const CAPACITY: usize = 64 * 1024 * 1024;

#[derive(Debug, Default)]
struct Stats {
    min: i16,
    max: i16,
    sum: i32,
    count: usize,
}


pub fn run(path: &Path) -> Result<String> {
    let file = File::open(path)?;
    let reader = BufReader::with_capacity(CAPACITY, file);

    let lines = reader.lines();

    let mut result: BTreeMap<String, Stats> = BTreeMap::new();

    for (i, line) in lines.enumerate() {
        if i % 1_000_000 == 0 {
            println!("Read {}M lines", i / 1_000_000);
        }

        let line = line?;
        let (name, value) = line.split_once(';').context("Invalid line")?;
        let value = parse_value(value)?;

        result
            .entry(name.to_owned())
            .and_modify(|stats| {
                if value < stats.min {
                    stats.min = value;
                }
                if value > stats.max {
                    stats.max = value;
                }
                stats.sum += value as i32;
                stats.count += 1;
            })
            .or_insert(Stats {
                min: value,
                max: value,
                sum: value as i32,
                count: 1,
            });
    }

    let mut output = String::new();
    write!(output, "{{")?;
    let mut first = true;

    for (name, stats) in result {
        let min = stats.min;
        let max = stats.max;

        let avg = average(stats.sum, stats.count)?;

        if !first {
            write!(output, ", ")?;
        }

        write!(
            output,
            "{}={}/{}/{}",
            name,
            Value(min),
            Value(avg),
            Value(max),
        )?;
        first = false;
    }
    writeln!(output, "}}")?;

    Ok(output)
}

struct Value(i16);

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

fn parse_value(s: &str) -> Result<i16, anyhow::Error> {
    let v = s.replace('.', "");
    let v = v.parse::<i16>().context("Invalid value")?;
    Ok(v)
}

#[inline]
fn average(sum: i32, count: usize) -> Result<i16, anyhow::Error> {
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

