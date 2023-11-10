use clap::Parser;
use polars::prelude::*;
use std::{
    fs::File,
    io::{self, BufReader},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_file: String,

    #[arg(short, long)]
    output_file: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let file = File::open(args.input_file)?;
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
        .expect("could not collect result of aggregation");

    if let Some(path) = args.output_file {
        let mut file = File::create(path).expect("unable to create output file");
        JsonWriter::new(&mut file)
            .with_json_format(JsonFormat::JsonLines)
            .finish(&mut result)
            .unwrap();
    } else {
        let mut stdout = std::io::stdout();
        JsonWriter::new(&mut stdout)
            .with_json_format(JsonFormat::JsonLines)
            .finish(&mut result)
            .unwrap();
    }

    Ok(())
}
