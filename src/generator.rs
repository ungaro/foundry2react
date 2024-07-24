use handlebars::Handlebars;
use serde_json::json;
use std::fs;
use std::path::Path;
use eyre::Result;

use crate::abi_parser::ContractFunction;
use crate::foundry_test_parser::TestCase;

pub fn generate_react_component(
    contract_functions: &[ContractFunction],
    test_cases: &[TestCase],
    output_path: &Path
) -> Result<()> {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("component", include_str!("../templates/react_component.hbs"))?;

    let data = json!({
        "functions": contract_functions.iter().map(|f| {
            json!({
                "name": f.name,
                "inputs": f.inputs.iter().map(|i| {
                    json!({
                        "name": i.name,
                        "type": i.type_,
                    })
                }).collect::<Vec<_>>(),
                "outputs": f.outputs.iter().map(|o| {
                    json!({
                        "name": o.name,
                        "type": o.type_,
                    })
                }).collect::<Vec<_>>(),
                "state_mutability": f.state_mutability,
            })
        }).collect::<Vec<_>>(),
        "test_cases": test_cases.iter().map(|tc| {
            json!({
                "name": tc.name,
                "function_calls": tc.function_calls.iter().map(|fc| {
                    json!({
                        "function_name": fc.function_name,
                        "arguments": fc.arguments,
                    })
                }).collect::<Vec<_>>(),
                "assertions": tc.assertions.iter().map(|a| {
                    json!({
                        "assertion_type": a.assertion_type,
                        "expected_value": a.expected_value,
                    })
                }).collect::<Vec<_>>(),
                "serialized": serde_json::to_string(tc).unwrap_or_default(),
            })
        }).collect::<Vec<_>>(),
    });

    let rendered = handlebars.render("component", &data)?;
    fs::write(output_path, rendered)?;

    Ok(())
}