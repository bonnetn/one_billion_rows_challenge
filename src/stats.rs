#![allow(dead_code)]

use anyhow::Result;
use std::{
    fmt::{Display, Formatter},
};

use anyhow::Context as _;
use rustc_hash::FxHashMap;


#[derive(Clone, Debug)]
pub struct StationStats {
    pub min: i16,
    pub max: i16,
    pub sum: i32,
    pub count: usize,
}

pub struct StationStatsMap(FxHashMap<Vec<u8>, StationStats>);

// As per the 1brc challenge, there is a maximum of 10,000 stations.
const STATS_CAPACITY: usize = 10_000;

impl StationStatsMap {
    pub fn new() -> Self {
        Self(FxHashMap::with_capacity_and_hasher(
            STATS_CAPACITY,
            Default::default(),
        ))
    }

    pub fn update(&mut self, station_name: &[u8], value: i16) {
        if let Some(stats) = self.0.get_mut(station_name) {
            stats.min = stats.min.min(value);
            stats.max = stats.max.max(value);
            stats.sum += value as i32;
            stats.count += 1;
        } else {
            self.0.insert(
                station_name.to_vec(),
                StationStats {
                    min: value,
                    max: value,
                    sum: value as i32,
                    count: 1,
                },
            );
        }
    }

    pub fn extend(&mut self, other: &StationStatsMap) {
        for (name, stats) in &other.0 {
            if let Some(existing_stats) = self.0.get_mut(name.as_slice()) {
                existing_stats.min = existing_stats.min.min(stats.min);
                existing_stats.max = existing_stats.max.max(stats.max);
                existing_stats.sum += stats.sum;
                existing_stats.count += stats.count;
            } else {
                self.0.insert(name.clone(), stats.clone());
            }
        }
    }

    pub fn report(&self) -> Result<Report<'_>> {
        let mut report = Report(Vec::new());
        for (name, stats) in &self.0 {
            let name = str::from_utf8(name.as_slice()).context("Invalid station name")?;
            let min = DecimalValue(stats.min);
            let avg = DecimalValue(average(stats.sum, stats.count));
            let max = DecimalValue(stats.max);
            report.0.push((name, min, avg, max));
        }

        report.0.sort_by_key(|(name, _, _, _)| *name);

        Ok(report)
    }
}

impl IntoIterator for StationStatsMap {
    type Item = (Vec<u8>, StationStats);
    type IntoIter = std::collections::hash_map::IntoIter<Vec<u8>, StationStats>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub struct Report<'a>(Vec<(&'a str, DecimalValue, DecimalValue, DecimalValue)>);

impl<'a> Display for Report<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{{")?;

        let mut first = true;
        for (name, min, avg, max) in &self.0 {
            if !first {
                write!(f, ", ")?;
            }

            write!(f, "{}={}/{}/{}", name, min, avg, max)?;
            first = false;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

#[inline]
pub fn average(sum: i32, count: usize) -> i16 {
    let count = count as i32;
    let avg = sum / count;
    let mut avg: i16 = avg.try_into().expect("Average is too large");
    let avg_rest = sum % count;

    if 2 * avg_rest >= count {
        avg += 1;
    }

    if 2 * avg_rest < -count {
        avg -= 1;
    }

    avg
}

pub struct DecimalValue(pub i16);

impl Display for DecimalValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let raw = self.0;
        let sign = if raw < 0 { "-" } else { "" };
        let abs = raw.abs();
        let int = abs / 10;
        let frac = abs % 10;

        write!(f, "{}{}.{}", sign, int, frac)
    }
}