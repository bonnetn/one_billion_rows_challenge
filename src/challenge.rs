use anyhow::Result;
use anyhow::{Context as _, bail};
use std::io::Read as _;
use std::thread;
use std::{fs::File, path::Path};

use crate::stats::StationStatsMap;
use crate::{station_name, value};

const CHUNK_SIZE: usize = 16 * 1024 * 1024;

struct WorkOrder {
    buffer: Vec<u8>,
    size: usize,
}

pub fn run(path: &Path) -> Result<String> {
    let concurrency = thread::available_parallelism()?.get();
    let total_stats = thread::scope(|s| {
        let (work_tx, work_rx) = std::sync::mpmc::sync_channel::<WorkOrder>(concurrency);
        let (buffer_tx, buffer_rx) = std::sync::mpsc::sync_channel::<Vec<u8>>(concurrency*2);

        // Pre-allocate the buffers.
        // We want to be able to have 2 buffers per worker:
        // - one actively being processed by the worker
        // - one waiting to be processed by the worker
        for _ in 0..(concurrency*2) {
            buffer_tx.send(vec![0; CHUNK_SIZE])?;
        }

        let feeder = std::thread::Builder::new()
            .name("feeder".to_owned())
            .spawn_scoped(s, move || {
                let mut file = File::open(path)?;
                let file_size: usize = file.metadata()?.len().try_into()?;
                let mut idx = 0_usize;
                let mut last_buffer = vec![];

                while idx < file_size {
                    let mut buffer = buffer_rx.recv()?;

                    let size = (file_size - idx).min(CHUNK_SIZE - last_buffer.len());

                    buffer[..last_buffer.len()].copy_from_slice(&last_buffer);

                    file.read_exact(&mut buffer[last_buffer.len()..size])?;
                    idx += size;

                    let mut truncated_size = size;
                    while truncated_size > 1 && buffer[truncated_size - 1] != b'\n' {
                        truncated_size -= 1;
                    }

                    last_buffer = buffer[truncated_size..size].to_vec();

                    work_tx.send(WorkOrder {
                        buffer,
                        size: truncated_size,
                    })?;
                }
                Ok::<_, anyhow::Error>(())
            })?;

        let handles = (0..concurrency)
            .map(|worker_id| {
                let work_rx = work_rx.clone();
                let buffer_tx = buffer_tx.clone();

                std::thread::Builder::new()
                    .name(format!("worker-{worker_id}"))
                    .spawn_scoped(s, move || {
                        let mut stats = StationStatsMap::new();
                        while let Ok(work_order) = work_rx.recv() {
                            process(&work_order.buffer[..work_order.size], &mut stats)?;
                            // println!("{worker_id}: Processing chunk of size {}", work_order.size);
                            if buffer_tx.send(work_order.buffer).is_err() {
                                // ignore error
                            }
                        }
                        Ok::<_, anyhow::Error>(stats)
                    })
                    .context("Failed to spawn worker")
            })
            .collect::<Result<Vec<_>>>()?;

        let stats_iter = handles
            .into_iter()
            .map(|handle| handle.join().expect("Failed to join worker"))
            .collect::<Result<Vec<_>>>()?;

        let mut total_stats = StationStatsMap::new();
        for stats in stats_iter {
            total_stats.extend(&stats);
        }

        feeder.join().expect("Failed to join feeder")?;

        Ok::<_, anyhow::Error>(total_stats)
    })?;

    let report = total_stats.report()?;

    let output = report.to_string();

    Ok(output)
}

fn process(input_data: &[u8], stats: &mut StationStatsMap) -> Result<()> {
    let mut data = input_data;

    while !data.is_empty() {
        let Some((station_name, rest)) = station_name::parse(data) else {
            let preview: String = String::from_utf8_lossy(&data[..data.len().min(80)]).into();
            bail!("Invalid line, no semicolon found (preview): {preview:?}");
        };

        let Some((value, rest)) = value::parse(rest) else {
            let preview: String = String::from_utf8_lossy(&rest[..rest.len().min(80)]).into();
            bail!("Invalid line, no value found (preview): {preview:?}");
        };

        let rest = strip_newline(rest);

        data = rest;

        stats.update(station_name, value);
    }

    Ok(())
}

#[inline]
fn strip_newline(d: &[u8]) -> &[u8] {
    match *d {
        [b'\n', ref rest @ ..] => rest,
        _ => d,
    }
}
