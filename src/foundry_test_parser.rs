use solang_parser::pt::{SourceUnit, SourceUnitPart, ContractPart, Statement, Expression};
use solang_parser::parse;
use eyre::{Result, eyre};
use std::fs;
use std::path::Path;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TestCase {
    pub name: String,
    pub function_calls: Vec<FunctionCall>,
    pub assertions: Vec<Assertion>,
}

#[derive(Debug, Serialize)]
pub struct FunctionCall {
    pub function_name: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Assertion {
    pub assertion_type: String,
    pub expected_value: String,
}


pub fn parse_foundry_test(path: &Path) -> Result<Vec<TestCase>> {
    let content = fs::read_to_string(path)
        .map_err(|e| eyre!("Failed to read Foundry test file: {}", e))?;

    let (source_unit, _) = parse(&content, 0)
        .map_err(|e| eyre!("Failed to parse Solidity content: {:?}", e))?;

    extract_test_cases(source_unit)
}


/*


pub fn parse_foundry_test(path: &Path) -> Result<Vec<TestCase>> {
    let content = fs::read_to_string(path)
        .map_err(|e| eyre!("Failed to read Foundry test file: {}", e))?;

    let (source_unit, _) = parse(&content, 0)
        .map_err(|e| eyre!("Failed to parse Solidity content: {:?}", e))?;

    extract_test_cases(source_unit)
}
*/
fn extract_test_cases(source_unit: SourceUnit) -> Result<Vec<TestCase>> {
    let mut test_cases = Vec::new();

    for part in source_unit.0 {
        if let SourceUnitPart::ContractDefinition(contract) = part {
            for part in contract.parts {
                if let ContractPart::FunctionDefinition(func) = part {
                    if func.name.as_ref().map_or(false, |name| name.name.starts_with("test")) {
                        let mut test_case = TestCase {
                            name: func.name.as_ref().map_or_else(String::new, |name| name.name.clone()),
                            function_calls: Vec::new(),
                            assertions: Vec::new(),
                        };

                        if let Some(Statement::Block { statements, .. }) = func.body {
                            for stmt in statements {
                                extract_function_calls_and_assertions(&stmt, &mut test_case);
                            }
                        }

                        test_cases.push(test_case);
                    }
                }
            }
        }
    }

    Ok(test_cases)
}

fn extract_function_calls_and_assertions(stmt: &Statement, test_case: &mut TestCase) {
    match stmt {
        Statement::Expression(_, Expression::FunctionCall(_, box_expr, args)) => {
            if let Expression::Variable(id) = box_expr.as_ref() {
                let function_name = id.name.clone();
                let arguments: Vec<String> = args.iter()
                    .map(|arg| format!("{:?}", arg))
                    .collect();
                
                if function_name == "assertTrue" || function_name == "assertEq" {
                    test_case.assertions.push(Assertion {
                        assertion_type: function_name,
                        expected_value: arguments.join(", "),
                    });
                } else {
                    test_case.function_calls.push(FunctionCall {
                        function_name,
                        arguments,
                    });
                }
            }
        },
        _ => {}
    }
}