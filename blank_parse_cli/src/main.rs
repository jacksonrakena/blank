use std::fs;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    rules_file: String,
}

fn main() {
    let args = Args::parse();

    let target_text = fs::read_to_string(args.rules_file.clone()).unwrap();
    let rules = blank_parse::parse_text(&*args.rules_file, &target_text).unwrap();
    for (_, rule) in &rules {
        println!("{:?}", rule);
    }
}