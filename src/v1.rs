#![allow(dead_code)]

use std::fmt::Write;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead as _, BufReader},
    path::Path,
};

use anyhow::{Context as _, Result};

use crate::stats::{Value, average};


#[derive(Debug, Default)]
struct Stats {
    min: i16,
    max: i16,
    sum: i32,
    count: usize,
}


pub fn run(path: &Path) -> Result<String> {
    let file = File::open(path)?;
    let reader = BufReader::with_capacity(1024 * 1024 * 1024, file);

    let lines = reader.lines();

    let mut result: BTreeMap<String, Stats> = BTreeMap::new();
    for (i, line) in lines.enumerate() {
        if i % 1_000_000 == 0  && i > 0 { 
            println!("Read {}M lines", i / 1_000_000);
        }

        let line = line?;
        let (name, value) = line.split_once(';').context("Invalid line")?;
        let value = parse_value(value)?;

        if let Some(stats) = result.get_mut(name) {
            if value < stats.min {
                stats.min = value;
            }
            if value > stats.max {
                stats.max = value;
            }
            stats.sum += value as i32;
            stats.count += 1;
        } else {
            result.insert(name.to_owned(), Stats {
                min: value,
                max: value,
                sum: value as i32,
                count: 1,
            });
        }
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


fn parse_value(s: &str) -> Result<i16, anyhow::Error> {
    let v = s.replace('.', "");
    let v = v.parse::<i16>().context("Invalid value")?;
    Ok(v)
}
