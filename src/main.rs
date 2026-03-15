use std::{path::Path, time::Instant};

use anyhow::Result;

mod generate;
mod station_name;
mod value;
mod stats;
mod challenge;

fn main() -> Result<()> {
    let start = Instant::now();
    challenge::run(Path::new("measurements.txt"))?;
    let end = Instant::now();
    println!("Time taken: {:?}", end.duration_since(start));
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

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

                    let got = challenge::run(&input_path).unwrap();
                    assert_eq!(
                        got,
                        expected,
                        "Invalid output: {}",
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
        let expected = std::fs::read_to_string(Path::new("measurements.out")).unwrap();
        let result = challenge::run(Path::new("measurements.txt")).unwrap();

        assert_eq!(result, expected);
    }
}
