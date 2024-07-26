use solang_parser::pt::{SourceUnit, SourceUnitPart, ContractPart, FunctionDefinition, Statement, Expression, VariableDefinition};
use solang_parser::parse;
use eyre::{eyre, Result, WrapErr};
use std::fs;
use std::path::Path;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TestContract {
    pub name: String,
    pub state_variables: Vec<StateVariable>,
    pub setup: Option<TestFunction>,
    pub test_functions: Vec<TestFunction>,
}

#[derive(Debug, Serialize)]
pub struct StateVariable {
    pub name: String,
    pub type_: String,
    pub value: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TestFunction {
    pub name: String,
    pub steps: Vec<TestStep>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum TestStep {
    VariableDeclaration {
        name: String,
        type_: String,
        value: Option<String>,
    },
    Constructor {
        contract: String,
        arguments: Vec<String>,
    },
    FunctionCall {
        contract: Option<String>,
        function: String,
        arguments: Vec<String>,
    },
    VMPrank(String),
    VMStartPrank(String),
    VMStopPrank,
    Assertion {
        assert_type: String,
        arguments: Vec<String>,
    },
}

pub fn parse_foundry_test_file(path: &Path) -> Result<TestContract> {
    let content = fs::read_to_string(path)
        .wrap_err("Failed to read Solidity test file")?;

    let (source_unit, _) = parse(&content, 0)
        .map_err(|e| eyre!("Failed to parse Solidity content: {:?}", e))?;

    extract_test_contract(source_unit)
}



fn extract_test_contract(source_unit: SourceUnit) -> Result<TestContract> {
    for part in source_unit.0 {
        if let SourceUnitPart::ContractDefinition(contract) = part {
            if contract.name.as_ref().map_or(false, |name| name.name.ends_with("Test")) {
                let mut state_variables = Vec::new();
                let mut setup = None;
                let mut test_functions = Vec::new();

                for part in &contract.parts {
                    match part {
                        ContractPart::VariableDefinition(var) => {
                            state_variables.push(extract_state_variable(var));
                        }
                        ContractPart::FunctionDefinition(func) => {
                            if func.name.as_ref().map_or(false, |name| name.name == "setUp") {
                                setup = Some(extract_function(func)?);
                            } else if is_test_function(func) {
                                test_functions.push(extract_function(func)?);
                            }
                        }
                        _ => {}
                    }
                }

                return Ok(TestContract {
                    name: contract.name.as_ref().map_or_else(String::new, |name| name.name.clone()),
                    state_variables,
                    setup,
                    test_functions,
                });
            }
        }
    }
    Err(eyre!("No test contract found"))
}

fn is_test_function(func: &FunctionDefinition) -> bool {
    func.name.as_ref().map_or(false, |name| 
        name.name.starts_with("test") || 
        name.name.starts_with("testFail") ||
        name.name.starts_with("testRevert")
    ) &&
    func.attributes.iter().any(|attr| matches!(attr, solang_parser::pt::FunctionAttribute::Visibility(solang_parser::pt::Visibility::Public(_))))
}

fn extract_function(func: &FunctionDefinition) -> Result<TestFunction> {
    let name = func.name.as_ref()
        .map(|ident| ident.name.clone())
        .ok_or_else(|| eyre!("Function has no name"))?;

    let mut steps = Vec::new();

    if let Some(Statement::Block { statements, .. }) = &func.body {
        for stmt in statements {
            if let Some(step) = extract_test_step(stmt) {
                steps.push(step);
            }
        }
    }

    Ok(TestFunction { name, steps })
}

fn extract_test_step(stmt: &Statement) -> Option<TestStep> {
    match stmt {
        Statement::Expression(_, expr) => extract_test_step_from_expression(expr),
        _ => None,
    }
}

fn extract_test_step_from_expression(expr: &Expression) -> Option<TestStep> {
    match expr {
        Expression::FunctionCall(_, box_expr, args) => {
            if let Expression::Variable(id) = box_expr.as_ref() {
                let function_name = id.name.clone();
                let arguments: Vec<String> = args.iter().map(|arg| format!("{:?}", arg)).collect();
                
                match function_name.as_str() {
                    "vm.prank" => Some(TestStep::VMPrank(arguments[0].clone())),
                    "vm.startPrank" => Some(TestStep::VMStartPrank(arguments[0].clone())),
                    "vm.stopPrank" => Some(TestStep::VMStopPrank),
                    "assertTrue" | "assertEq" => Some(TestStep::Assertion {
                        assert_type: function_name,
                        arguments,
                    }),
                    _ => Some(TestStep::FunctionCall {
                        contract: None,
                        function: function_name,
                        arguments,
                    }),
                }
            } else if let Expression::MemberAccess(_, box_expr, member) = box_expr.as_ref() {
                if let Expression::Variable(id) = box_expr.as_ref() {
                    let contract = id.name.clone();
                    let function = member.name.clone();
                    let arguments: Vec<String> = args.iter().map(|arg| format!("{:?}", arg)).collect();
                    Some(TestStep::FunctionCall {
                        contract: Some(contract),
                        function,
                        arguments,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        },
        _ => None,
    }
}


fn extract_state_variable(var: &VariableDefinition) -> StateVariable {
    StateVariable {
        name: var.name.as_ref().map_or_else(String::new, |id| id.name.clone()),
        type_: format!("{:?}", var.ty),
        value: var.initializer.as_ref().map(|expr| format!("{:?}", expr)),
    }
}
