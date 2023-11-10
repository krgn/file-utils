use clap::Parser;
use polars::prelude::*;
use rayon::prelude::ParallelIterator;
use std::{
    fs::File,
    io::{self, BufReader},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: Option<String>,
}

fn stdin_connected() -> bool {
    atty::isnt(atty::Stream::Stdin)
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    if args.file.is_none() && !stdin_connected() {
        eprintln!("Error: stdin not used nor --file was specified");
        std::process::exit(1)
    }

    if args.file.is_none() {
        eprintln!("Error: streaming not implemented and --file was not specified");
        std::process::exit(1)
    }

    let file = File::open(args.file.unwrap())?;
    let reader = BufReader::new(file);
    let schema = Schema::from_iter(vec![
        Field::new("file_path", DataType::Utf8),
        Field::new("hash", DataType::Utf8),
    ]);

    let df = JsonReader::new(reader)
        .with_json_format(JsonFormat::JsonLines)
        .with_schema(schema.into())
        .finish()
        .expect("could not build data frame")
        .lazy();

    let mut result = df
        .group_by([col("hash")])
        .agg([
            col("file_path").alias("files"),
            col("file_path").count().alias("count"),
        ])
        .filter(col("count").gt(1))
        .collect()
        .expect("boom");

    let mut outfile = File::create("out.json").expect("unable to create out.json");
    JsonWriter::new(&mut outfile)
        .with_json_format(JsonFormat::JsonLines)
        .finish(&mut result)
        .unwrap();

    Ok(())
}
