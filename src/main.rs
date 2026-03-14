use std::path::Path;

use anyhow::Result;

mod generate;
mod v1;

fn main() -> Result<()> {
    // generate::generate_samples()?;
    let path = Path::new("measurements.txt");
    let result = v1::run(path)?;
    println!("{}", result);
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    const BASE_PATH: &str = "1brc/src/test/resources/samples";

    const SAMPLES: &[&str] = &[
        "measurements-1",
        "measurements-10",
        "measurements-10000-unique-keys",
        "measurements-2",
        "measurements-20",
        "measurements-3",
        "measurements-boundaries",
        "measurements-complex-utf8",
        "measurements-dot",
        "measurements-rounding",
        "measurements-short",
        "measurements-shortest",
    ];

    #[test]
    fn test_samples() {
        for sample in SAMPLES {
            let input_path = Path::new(BASE_PATH).join(sample).with_extension("txt");
            let output_path = Path::new(BASE_PATH).join(sample).with_extension("out");
            println!("Testing {}", input_path.display());

            let result = v1::run(&input_path).unwrap();

            let expected = std::fs::read_to_string(output_path).unwrap();
            assert_eq!(
                result,
                expected,
                "Invalid output for {}",
                input_path.display()
            );
        }
    }
}