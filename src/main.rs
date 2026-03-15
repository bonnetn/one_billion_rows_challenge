use std::{path::Path, time::Instant};

use anyhow::Result;

mod generate;
mod station_name;
mod value;
mod stats;
mod v1;
mod v2;
mod v3;

fn main() -> Result<()> {
    let start = Instant::now();
    v3::run(Path::new("measurements.txt"))?;
    let end = Instant::now();
    println!("Time taken: {:?}", end.duration_since(start));
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;
    use std::path::Path;

    use serde::Deserialize;
    use serde_json::Deserializer;

    use super::*;

    const BASE_PATH: &str = "1brc/src/test/resources/samples";

    macro_rules! sample_tests {
        ($( $name:ident => $sample:expr ),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let base_path = std::path::Path::new(BASE_PATH);

                    let input_path = base_path
                        .join($sample)
                        .with_extension("txt");

                    let output_path = base_path
                        .join($sample)
                        .with_extension("out");

                    let expected = std::fs::read_to_string(&output_path).unwrap();

                    let result_v1 = v1::run(&input_path).unwrap();
                    assert_eq!(
                        result_v1,
                        expected,
                        "Invalid output for v1: {}",
                        input_path.display()
                    );

                    let result_v2 = v2::run(&input_path).unwrap();
                    assert_eq!(
                        result_v2,
                        expected,
                        "Invalid output for v2: {}",
                        input_path.display()
                    );

                    let result_v3 = v3::run(&input_path).unwrap();
                    assert_eq!(
                        result_v3,
                        expected,
                        "Invalid output for v3: {}",
                        input_path.display()
                    );
                }
            )*
        };
    }

    sample_tests!(
        measurements_1 => "measurements-1",
        measurements_10 => "measurements-10",
        measurements_10000_unique_keys => "measurements-10000-unique-keys",
        measurements_2 => "measurements-2",
        measurements_20 => "measurements-20",
        measurements_3 => "measurements-3",
        measurements_boundaries => "measurements-boundaries",
        measurements_complex_utf8 => "measurements-complex-utf8",
        measurements_dot => "measurements-dot",
        measurements_rounding => "measurements-rounding",
        measurements_short => "measurements-short",
        measurements_shortest => "measurements-shortest",
    );

    #[test]
    fn test_measurements() {
        let expected = get_expected_output().unwrap();
        let result = v2::run(Path::new("measurements.txt")).unwrap();

        assert_eq!(result, expected);
    }

    fn get_expected_output() -> Result<String> {
        #[derive(Debug, Deserialize)]
        struct Measurement {
            station: String,
            min: f64,
            avg: f64,
            max: f64,
        }

        let output_path = Path::new("measurements.out.json");

        let expected = std::fs::read_to_string(&output_path)?;
        let mut measurements = Deserializer::from_str(&expected)
            .into_iter::<Measurement>()
            .collect::<Result<Vec<_>, _>>()?;

        measurements.sort_by_key(|m| m.station.clone());

        let mut output = String::new();
        write!(output, "{{")?;
        let mut first = true;
        for measurement in measurements {
            if !first {
                write!(output, ", ")?;
            }
            first = false;
            let min = (measurement.min * 10.0).round() / 10.0;
            let avg = (measurement.avg * 10.0).round() / 10.0;
            let max = (measurement.max * 10.0).round() / 10.0;
            write!(
                output,
                "{}={:.1}/{:.1}/{:.1}",
                measurement.station, min, avg, max
            )?;
        }
        writeln!(output, "}}")?;

        Ok(output)
    }
}
