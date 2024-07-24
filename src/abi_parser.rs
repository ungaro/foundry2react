use serde_json::Value;
use eyre::{eyre, Result, WrapErr};
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct ContractFunction {
    pub name: String,
    pub inputs: Vec<FunctionParameter>,
    pub outputs: Vec<FunctionParameter>,
    pub state_mutability: String,
}

#[derive(Debug)]
pub struct FunctionParameter {
    pub name: String,
    pub type_: String,
}

pub fn parse_abi(path: &Path) -> Result<Vec<ContractFunction>> {
    let content = fs::read_to_string(path)
        .wrap_err("Failed to read ABI file")?;

    let abi: Value = serde_json::from_str(&content)
        .wrap_err("Failed to parse ABI JSON")?;

    let abi_array = abi.as_array()
        .ok_or_else(|| eyre!("ABI is not an array"))?;

    abi_array.iter()
        .filter_map(|item| {
            if item["type"] == "function" {
                Some(parse_function(item))
            } else {
                None
            }
        })
        .collect::<Result<Vec<_>>>()
}

fn parse_function(function: &Value) -> Result<ContractFunction> {
    Ok(ContractFunction {
        name: function["name"].as_str().ok_or_else(|| eyre!("Function name not found"))?.to_string(),
        inputs: parse_parameters(&function["inputs"])?,
        outputs: parse_parameters(&function["outputs"])?,
        state_mutability: function["stateMutability"].as_str().ok_or_else(|| eyre!("State mutability not found"))?.to_string(),
    })
}

fn parse_parameters(params: &Value) -> Result<Vec<FunctionParameter>> {
    params.as_array()
        .ok_or_else(|| eyre!("Parameters are not an array"))?
        .iter()
        .map(|param| {
            Ok(FunctionParameter {
                name: param["name"].as_str().ok_or_else(|| eyre!("Parameter name not found"))?.to_string(),
                type_: param["type"].as_str().ok_or_else(|| eyre!("Parameter type not found"))?.to_string(),
            })
        })
        .collect()
}