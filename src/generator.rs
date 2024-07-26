use handlebars::Handlebars;
use serde_json::json;
use std::fs;
use std::path::Path;

use std::fs::OpenOptions;
use std::io::Write;

use eyre::Result;



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

fn parse_function_call_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
    let function_call = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let parsed = parse_function_call(function_call);
    out.write(&parsed)?;
    Ok(())
}




fn parse_function_call(func_call: &str) -> String {
    if func_call.contains("FunctionCall(") {
        let function_name = extract_function_name(func_call);
        let args = extract_function_args(func_call);
        if function_name == "transfer" || function_name == "approve" || function_name == "transferFrom" {
            format!("contract.write.{}([{}])", function_name, args.join(", "))
        } else {
            format!("contract.read.{}([{}])", function_name, args.join(", "))
        }
    } else {
        func_call.to_string()
    }
}




fn parse_arg_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    log_debug(&format!("parse_arg_helper input: {}", param));
    let parsed = parse_argument(param);
    log_debug(&format!("parse_arg_helper output: {}", parsed));
    out.write(&parsed)?;
    Ok(())
}
fn parse_argument(arg: &str) -> String {
    if arg.contains("FunctionCall(") {
        parse_function_call(arg)
    } else if arg.contains("Variable(") {
        if let Some(start) = arg.rfind("name: \"") {
            let start = start + 7;
            if let Some(end) = arg[start..].find('"') {
                return arg[start..start+end].to_string();
            }
        }
        arg.to_string()
    } else if arg.contains("NumberLiteral(") {
        if let Some(start) = arg.rfind(", \"") {
            let start = start + 3;
            if let Some(end) = arg[start..].find('"') {
                return format!("BigInt({}e18)", &arg[start..start+end]);
            }
        }
        arg.to_string()
    } else {
        arg.trim().to_string()
    }
}

fn extract_function_name(func_call: &str) -> String {
    // Look for the last occurrence of 'Identifier'
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

fn extract_function_args(func_call: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut depth = 0;
    let mut current_arg = String::new();
    
    for c in func_call.chars() {
        match c {
            '[' => {
                depth += 1;
                if depth > 1 {
                    current_arg.push(c);
                }
            },
            ']' => {
                depth -= 1;
                if depth == 0 {
                    if !current_arg.is_empty() {
                        args.push(parse_argument(&current_arg));
                        current_arg.clear();
                    }
                } else {
                    current_arg.push(c);
                }
            },
            ',' if depth == 1 => {
                if !current_arg.is_empty() {
                    args.push(parse_argument(&current_arg));
                    current_arg.clear();
                }
            },
            _ if depth > 0 => current_arg.push(c),
            _ => {}
        }
    }
    
    args
}

///
fn parse_assertion_function_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
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



fn parse_assertion(arg: &str) -> String {
    if arg.contains("FunctionCall(") {
        let function = extract_function_name(arg);
        let args = extract_function_args(arg);
        format!("await contract.read.{}([{}])", function, args.join(", "))
    } else {
        parse_argument(arg)
    }
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
