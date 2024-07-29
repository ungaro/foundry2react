use eyre::Result;
use clap::Parser;
use std::path::PathBuf;
use std::fs;

mod abi_parser;
mod foundry_test_parser;
mod generator;

#[derive(Parser)]
#[clap(version = "1.0", author = "Alp Guneysel")]
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

    let test_contract = if let Some(test_path) = opts.test {
        foundry_test_parser::parse_foundry_test_file(&test_path)?
    } else {
        println!("No test file provided. Skipping test parsing.");
        return Ok(());
    };

    println!("====================================");
    println!("Test Contract: {}", test_contract.name);
    
    println!("\nState Variables:");
    for var in &test_contract.state_variables {
        println!("  {}: {} = {:?}", var.name, var.type_, var.value);
    }

    if let Some(setup) = &test_contract.setup {
        println!("\nSetup Function:");
        for step in &setup.steps {
            println!("  {:?}", step);
        }
    }

    println!("\nTest Functions:");
    for func in &test_contract.test_functions {
        println!("\nTest: {}", func.name);
        for step in &func.steps {
            println!("  {:?}", step);
        }
    }


    let js_code = generator::generate_js_code(&test_contract)?;

    fs::write(&opts.output, js_code)?;

    println!("\nReact component generation somewhat implemented");
    Ok(())
}