use eyre::Result;
use clap::Parser;
use std::path::PathBuf;

mod abi_parser;
mod foundry_test_parser;
mod generator;

#[derive(Parser)]
#[clap(version = "1.0", author = "Your Name")]
struct Opts {
    #[clap(short, long)]
    abi: PathBuf,
    
    #[clap(short, long)]
    test: Option<PathBuf>,
    
    #[clap(short, long)]
    output: PathBuf,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let contract_functions = abi_parser::parse_abi(&opts.abi)?;
    
    let test_cases = if let Some(test_path) = opts.test {
        foundry_test_parser::parse_foundry_test(&test_path)?
    } else {
        vec![]
    };

    generator::generate_react_component(&contract_functions, &test_cases, &opts.output)?;

    println!("React component generated successfully!");
    Ok(())
}