use solang_parser::pt::{SourceUnit, ContractPart, Statement, Expression};
use solang_parser::parsers::parse_raw_source;
use anyhow::{Result, Context};
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct TestCase {
    pub name: String,
    pub function_calls: Vec<FunctionCall>,
    pub assertions: Vec<Assertion>,
}

#[derive(Debug)]
pub struct FunctionCall {
    pub function_name: String,
    pub arguments: Vec<String>,
}

#[derive(Debug)]
pub struct Assertion {
    pub assertion_type: String,
    pub expected_value: String,
}

pub fn parse_foundry_test(path: &Path) -> Result<Vec<TestCase>> {
    let content = fs::read_to_string(path)
        .context("Failed to read Foundry test file")?;

    let (source_unit, _) = parse_raw_source(&content, 0)
        .map_err(|e| anyhow::anyhow!("Failed to parse Solidity content: {:?}", e))?;

    extract_test_cases(source_unit)
}

fn extract_test_cases(source_unit: SourceUnit) -> Result<Vec<TestCase>> {
    let mut test_cases = Vec::new();

    for part in source_unit.0 {
        if let SourceUnit::ContractDefinition(contract) = part {
            for part in contract.parts {
                if let ContractPart::FunctionDefinition(func) = part {
                    if func.name.as_ref().map_or(false, |name| name.name.starts_with("test")) {
                        let mut test_case = TestCase {
                            name: func.name.as_ref().map_or_else(String::new, |name| name.name.clone()),
                            function_calls: Vec::new(),
                            assertions: Vec::new(),
                        };

                        if let Statement::Block { statements, .. } = func.body.as_ref().context("Function body not found")? {
                            for stmt in statements {
                                extract_function_calls_and_assertions(stmt, &mut test_case);
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
        Statement::Expression(_, Expression::FunctionCall { function, arguments, .. }) => {
            if let Expression::Variable(id) = function.as_ref() {
                let function_name = id.name.clone();
                let args: Vec<String> = arguments.iter()
                    .map(|arg| format!("{:?}", arg))
                    .collect();
                test_case.function_calls.push(FunctionCall {
                    function_name,
                    arguments: args,
                });
            }
        },
        Statement::Expression(_, Expression::FunctionCall { function, arguments, .. }) => {
            if let Expression::Variable(id) = function.as_ref() {
                if id.name == "assertTrue" || id.name == "assertEq" {
                    test_case.assertions.push(Assertion {
                        assertion_type: id.name.clone(),
                        expected_value: format!("{:?}", arguments[0]),
                    });
                }
            }
        },
        _ => {}
    }
}