use std::collections::HashMap;
use walkdir::WalkDir;
use blank_parse::parse_text;
use blank_parse::rules::Rule;

#[test]
fn run_or_record_sample_tests() {
    use std::fs;
    use std::path::Path;

    let samples_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("samples");
    let recordings_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/recordings");

    let walked = WalkDir::new(&samples_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();

    println!("Loaded {} samples from {:?}", walked.len(), samples_dir);
    for entry in walked {
        let path = entry.path();
        if path.is_file() {
            let relative_path = path.strip_prefix(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("..")
                    .join("samples"),
            ).unwrap();
            let recording_path = recordings_dir.join(relative_path);
            fs::create_dir_all(recording_path.parent().unwrap()).unwrap();
            let output = parse_text(path.to_str().unwrap(), &fs::read_to_string(path).unwrap());

            let output_str = format_output(output);
            if recording_path.exists() {
                let recorded_output = fs::read_to_string(&recording_path).unwrap();
                assert_eq!(
                    output_str, recorded_output,
                    "Output for sample {:?} does not match recorded output",
                    relative_path
                );
            } else {
                fs::write(&recording_path, output_str).unwrap();
                println!(
                    "Recorded output for sample {:?} to {:?}",
                    relative_path, recording_path
                );
            }
        }
    }
}


fn format_output<E: std::fmt::Debug>(input: Result<HashMap<String, Rule>, E>) -> String {
    match input {
        Ok(rules) => {
            let mut r =  rules.values().collect::<Vec<&Rule>>();
            r.sort_by(|a,b| a.name.cmp(&b.name));
            format!("{:?}", r)
        }
        Err(_) => {
            format!("{:?}",input)
        }
    }
}