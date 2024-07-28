use handlebars::Handlebars;
use serde_json::json;
use std::fs;
use std::path::Path;

use std::fs::OpenOptions;
use std::io::Write;

use eyre::Result;

use std::collections::VecDeque;
use serde_json::Value;
use handlebars::{ RenderErrorReason, RenderError};


fn log_debug(message: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug_log.txt")
        .unwrap();
    writeln!(file, "{}", message).unwrap();
}

//mod foundry_test_parser;
//use crate::foundry_test_parser::TestContract;
use crate::foundry_test_parser::{TestContract, TestStep};

pub fn generate_js_code(test_contract: &TestContract) -> Result<String, handlebars::RenderError> {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("component", include_str!("../templates/react_component.hbs"))?;
    
    handlebars.register_helper("capitalize", Box::new(capitalize_helper));
    handlebars.register_helper("uppercase", Box::new(uppercase_helper));
    handlebars.register_helper("parseArg", Box::new(parse_arg_helper));
    handlebars.register_helper("parseAssertionFunction", Box::new(parse_assertion_function_helper));
    handlebars.register_helper("parseAssertionArgs", Box::new(parse_assertion_args_helper));
    handlebars.register_helper("generateAssertionVar", Box::new(generate_assertion_var_helper));
    handlebars.register_helper("parseAssertion", Box::new(parse_assertion_helper));
    handlebars.register_helper("parseFunctionCall", Box::new(parse_function_call_helper));
    handlebars.register_helper("raw", Box::new(raw_helper));
    handlebars.register_helper("json", Box::new(json_helper));  // Add this line


    let contract_functions = extract_contract_functions(test_contract);

    let data = json!({
        "contractName": test_contract.name,
        "stateVariables": test_contract.state_variables,
        "setupFunction": test_contract.setup,
        "testFunctions": test_contract.test_functions,
        "contractFunctions": contract_functions,
    });

    handlebars.render("component", &data)
}



fn raw_helper(
    h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output
) -> handlebars::HelperResult {
    let param = h.param(0).ok_or_else(|| RenderError::new("Param 0 is required for raw helper"))?;
    let value = param.value();
    
    out.write("{")?;
    match value {
        Value::String(s) => out.write(s.as_str())?,
        _ => out.write(&value.to_string())?,
    }
    out.write("}")?;
    Ok(())
}

fn capitalize_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let capitalized = param.chars().next().map(|c| c.to_uppercase().collect::<String>() + &param[1..]).unwrap_or_default();
    out.write(&capitalized)?;
    Ok(())
}

fn uppercase_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    out.write(&param.to_uppercase())?;
    Ok(())
}

fn parse_assertion_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
    let arg = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let parsed = parse_assertion(arg);
    out.write(&parsed)?;
    Ok(())
}



fn parse_function_call(map: &serde_json::Map<String, Value>) -> String {
    let empty_map = serde_json::Map::new();
    let empty_vec = Vec::new();

    let function = map.get("function").and_then(|f| f.as_object()).unwrap_or(&empty_map);
    let function_name = function.get("name").and_then(|n| n.as_str()).unwrap_or("unknownFunction");
    
    let arguments = map.get("arguments").and_then(|a| a.as_array()).unwrap_or(&empty_vec);
    let parsed_args: Vec<String> = arguments.iter().map(|a| parse_argument(a)).collect();
    
    if function_name == "transfer" || function_name == "approve" || function_name == "transferFrom" {
        format!("contract.write.{}({})", function_name, parsed_args.join(", "))
    } else {
        format!("await contract.read.{}({})", function_name, parsed_args.join(", "))
    }
}

fn parse_variable(map: &serde_json::Map<String, Value>) -> String {
    map.get("name").and_then(|n| n.as_str()).unwrap_or("unknownVariable").to_string()
}

fn parse_number_literal(map: &serde_json::Map<String, Value>) -> String {
    let value = map.get("value").and_then(|v| v.as_str()).unwrap_or("0");
    format!("BigInt({}e18)", value)
}


fn extract_variable_name(var: &str) -> String {
    if let Some(start) = var.rfind("name: \"") {
        let start = start + 7;
        if let Some(end) = var[start..].find('"') {
            return var[start..start+end].to_string();
        }
    }
    "unknownVariable".to_string()
}

fn extract_number_literal(num: &str) -> String {
    if let Some(start) = num.rfind(", \"") {
        let start = start + 3;
        if let Some(end) = num[start..].find('"') {
            return format!("BigInt({}e18)", &num[start..start+end]);
        }
    }
    "BigInt(0)".to_string()
}
/*
fn parse_arg_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    log_debug(&format!("parse_arg_helper input: {}", param));
    let parsed = parse_ast(param);
    log_debug(&format!("parse_arg_helper output: {}", parsed));
    out.write(&parsed)?;
    Ok(())
}
*/
fn parse_arg_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
    let param = h.param(0).ok_or_else(|| RenderError::new("Param 0 is required for parseArg helper"))?;
    let parsed = parse_assertion(param.value().as_str().unwrap_or(""));
    out.write(&parsed)?;
    Ok(())
}

/*

fn parse_function_call_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
    let function_call = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    log_debug(&format!("parse_function_call_helper input: {}", function_call));
    let parsed = parse_ast(function_call);
    log_debug(&format!("parse_function_call_helper output: {}", parsed));
    out.write(&parsed)?;
    Ok(())
}
*/

fn parse_function_call_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
    let param = h.param(0).ok_or_else(|| RenderError::new("Param 0 is required for parseFunctionCall helper"))?;
    let parsed = parse_assertion(param.value().as_str().unwrap_or(""));
    out.write(&parsed)?;
    Ok(())
}


pub fn parse_ast(ast: &str) -> String {
    log_debug(&format!("Parsing AST: {}", ast));
    let v: Value = match serde_json::from_str(ast) {
        Ok(v) => v,
        Err(e) => {
            log_debug(&format!("Failed to parse JSON: {}", e));
            return "null".to_string();
        }
    };
    parse_value(&v)
}


fn parse_value(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            if let Some(typ) = map.get("type") {
                match typ.as_str() {
                    Some("FunctionCall") => parse_function_call(map),
                    Some("Variable") => parse_variable(map),
                    Some("NumberLiteral") => parse_number_literal(map),
                    Some(t) => {
                        log_debug(&format!("Unknown type: {}", t));
                        value.to_string()
                    },
                    None => {
                        log_debug("Type is not a string");
                        value.to_string()
                    }
                }
            } else {
                log_debug("No type field found");
                value.to_string()
            }
        },
        Value::Array(arr) => {
            let parsed: Vec<String> = arr.iter().map(parse_value).collect();
            format!("[{}]", parsed.join(", "))
        },
        _ => value.to_string(),
    }
}


fn parse_argument(arg: &Value) -> String {
    match arg {
        Value::Object(map) => {
            if let Some(typ) = map.get("type") {
                match typ.as_str() {
                    Some("FunctionCall") => parse_function_call(map),
                    Some("Variable") => parse_variable(map),
                    Some("NumberLiteral") => parse_number_literal(map),
                    Some(t) => {
                        log_debug(&format!("Unknown type in parse_argument: {}", t));
                        arg.to_string()
                    },
                    None => {
                        log_debug("Type is not a string in parse_argument");
                        arg.to_string()
                    }
                }
            } else {
                log_debug("No type field found in parse_argument");
                arg.to_string()
            }
        },
        Value::Array(arr) => {
            let parsed: Vec<String> = arr.iter().map(|v| parse_argument(v)).collect();
            format!("[{}]", parsed.join(", "))
        },
        Value::String(s) => format!("\"{}\"", s),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
    }
}


fn extract_function_name(func_call: &str) -> String {
    if let Some(last_identifier) = func_call.rfind("Identifier") {
        if let Some(start) = func_call[last_identifier..].find("name: \"") {
            let start = last_identifier + start + 7;
            if let Some(end) = func_call[start..].find('"') {
                return func_call[start..start+end].to_string();
            }
        }
    }
    "unknownFunction".to_string()
}


//HERE IS THE BUG
fn extract_function_args(func_call: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut depth = 0;
    let mut in_brackets = false;

    for c in func_call.chars() {
        match c {
            '[' => {
                in_brackets = true;
                depth += 1;
                current_arg.push(c);
            },
            ']' => {
                depth -= 1;
                current_arg.push(c);
                if depth == 0 {
                    in_brackets = false;
                    if !current_arg.is_empty() {
                        args.push(current_arg.trim().to_string());
                        current_arg.clear();
                    }
                }
            },
            ',' if in_brackets && depth == 1 => {
                if !current_arg.is_empty() {
                    args.push(current_arg.trim().to_string());
                    current_arg.clear();
                }
            },
            _ if in_brackets => current_arg.push(c),
            _ => {}
        }
    }

    args
}

///
fn parse_assertion_function_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
    println!("parse_assertion_function_helper:");
    println!("handlebars: {:#?}",h);
    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let function = extract_function_name(param);
    out.write(&function)?;
    Ok(())
}

fn parse_assertion_args_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let args = extract_function_args(param);
    out.write(&args.join(", "))?;
    Ok(())
}

fn generate_assertion_var_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let function = extract_function_name(param);
    let var_name = format!("{}Result", function);
    out.write(&var_name)?;
    Ok(())
}

fn json_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {

    let param = h.param(0).ok_or_else(|| RenderError::new("Param 0 is required for json helper"))?;
    let json_str = serde_json::to_string(param.value())
        .map_err(|e| RenderError::new(format!("Failed to serialize to JSON: {}", e)))?;
    out.write(&json_str)?;
    Ok(())
}


fn parse_assertion(arg: &str) -> String {
    let v: Value = match serde_json::from_str(arg) {
        Ok(v) => v,
        Err(e) => {
            log_debug(&format!("Failed to parse JSON in parse_assertion: {}", e));
            return "null".to_string();
        }
    };

    parse_argument(&v)
}


fn extract_contract_functions(test_contract: &TestContract) -> Vec<String> {
    let mut functions = vec![];
    if let Some(setup) = &test_contract.setup {
        extract_functions_from_steps(&setup.steps, &mut functions);
    }
    for test_function in &test_contract.test_functions {
        extract_functions_from_steps(&test_function.steps, &mut functions);
    }
    functions.sort();
    functions.dedup();
    functions
}

fn extract_functions_from_steps(steps: &[TestStep], functions: &mut Vec<String>) {
    for step in steps {
        if let TestStep::FunctionCall { function, arguments, .. } = step {
            let args = arguments.iter().map(|_| "address".to_string()).collect::<Vec<_>>().join(", ");
            functions.push(format!("function {}({}) public", function, args));
        }
    }
}
