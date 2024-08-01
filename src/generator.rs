use handlebars::Handlebars;
use serde_json::json;
use std::fs;
use std::path::Path;

use std::fs::OpenOptions;
use std::io::Write;
use regex::Regex;
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
    log_debug(&format!("PARAM: {}", param));
    let capitalized = param.chars().next().map(|c| c.to_uppercase().collect::<String>() + &param[1..]).unwrap_or_default();
    log_debug(&format!("PARAM_CAP: {}", capitalized));
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

println!("function_call: {:#?}",map);

    let function = parse_value(map.get("function").unwrap_or(&Value::Null));
    let empty_vec = Vec::new();
    let arguments = map.get("arguments").and_then(|a| a.as_array()).unwrap_or(&empty_vec);
    let parsed_args: Vec<String> = arguments.iter().map(parse_value).collect();
    format!("{}({})", function, parsed_args.join(", "))


}


fn parse_member_access(map: &serde_json::Map<String, Value>) -> String {
    let object = parse_value(map.get("object").unwrap_or(&Value::Null));
    let member = parse_value(map.get("member").unwrap_or(&Value::Null));
    format!("{}.{}", object, member)
}






fn parse_variable(map: &serde_json::Map<String, Value>) -> String {
    map.get("name")
       .and_then(|n| n.as_object())
       .and_then(|o| o.get("name"))
       .and_then(|n| n.as_str())
       .unwrap_or("null")
       .to_string()
}


fn parse_number_literal(map: &serde_json::Map<String, Value>) -> String {
    let value = map.get("value").and_then(|v| v.as_str()).unwrap_or("0");
    let subdenomination = map.get("subdenomination").and_then(|s| s.as_str()).unwrap_or("");
    format!("BigInt({}{})", value, subdenomination)
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

fn parse_identifier(map: &serde_json::Map<String, Value>) -> String {
    map.get("name")
       .and_then(|n| n.as_str())
       .unwrap_or("null")
       .to_string()
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

fn parse_arg_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
    let param = h.param(0).ok_or_else(|| RenderError::new("Param 0 is required for parseArg helper"))?;
    let input = param.value().to_string();
    log_debug(&format!("parse_arg_helper input: {}", input));
    
    let parsed = parse_ast_string(&input);
    log_debug(&format!("parse_arg_helper parsed: {}", parsed));

    out.write(&parsed)?;
    Ok(())
}

fn parse_ast_string(input: &str) -> String {
    let trimmed = input.trim_matches('"');
    if trimmed.starts_with("FunctionCall(") {
        parse_function_call_string(trimmed)
    } else if trimmed.starts_with("MemberAccess(") {
        parse_member_access_string(trimmed)
    } else if trimmed.starts_with("Variable(") {
        parse_variable_string(trimmed)
    } else if trimmed.starts_with("NumberLiteral(") {
        parse_number_literal_string(trimmed)
    } else if trimmed.starts_with("Identifier(") {
        parse_identifier_string(trimmed)
    } else {
        log_debug(&format!("Unknown AST node type: {}", trimmed));
        trimmed.to_string()
    }
}

fn parse_identifier_string(input: &str) -> String {
    if let Some(start) = input.rfind("name: \"") {
        let start = start + 7;
        if let Some(end) = input[start..].find('"') {
            return input[start..start+end].to_string();
        }
    }
    log_debug(&format!("Failed to parse identifier: {}", input));
    "unknownIdentifier".to_string()
}


    fn parse_function_call_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
        let param = h.param(0).ok_or_else(|| RenderError::new("Param 0 is required for parseFunctionCall helper"))?;
        let input = param.value().to_string();
        log_debug(&format!("parse_function_call_helper input: {}", input));
        
        let parsed = parse_ast_string(&input);
        log_debug(&format!("parse_function_call_helper parsed: {}", parsed));
    
        out.write(&parsed)?;
        Ok(())
    }
/*
    fn parse_function_call_string(input: &str) -> String {
        log_debug(&format!("parse_function_call_string input: {}", input));

        let inner = input.trim_start_matches("FunctionCall(").trim_end_matches(')');
        let parts: Vec<&str> = split_top_level(inner, ", ");
        log_debug(&format!("parse_function_call_string parts: {:#?}", parts));

        let result = if parts.len() >= 3 {
            let member_access = parse_ast_string(parts[1]);
            let args = parse_arguments_string(parts[2]);
            format!("{}({})", member_access, args)
        } else {
            "null".to_string()
        };
        log_debug(&format!("parse_function_call_string output: {}", result));

        result
    }


fn parse_function_call_string(input: &str) -> String {
    let parts: Vec<&str> = split_top_level(input, ", ");
    log_debug(&format!("Failed to parse function call / parts: {:#?}", parts));

    if parts.len() >= 3 {
        let function = parse_ast_string(parts[1]);
        let args = parse_arguments_string(parts[2]);
        if function.contains('.') {
            format!("{}({})", function, args)
        } else {
            format!("contract.write.{}({})", function, args)
        }
    } else {
        log_debug(&format!("Failed to parse function call: {}", input));
        "unknownFunction11()".to_string()
    }
}
*/
/*
fn parse_function_call_string(input: &str) -> String {
    let content = input.trim_start_matches("FunctionCall(").trim_end_matches(')');
    let parts: Vec<&str> = split_top_level(content, ", ");
    
    if parts.len() >= 3 {
        let _file_info = parts[0]; // We're ignoring this for now
        let function = parse_ast_string(parts[1]);
        let args = parse_arguments_string(parts[2]);
        format!("{}({})", function, args)
    } else {
        log_debug(&format!("Unexpected FunctionCall format: {}", input));
        "unknownFunction()".to_string()
    }
}
*/
fn parse_function_call_string(input: &str) -> String {
    let content = input.trim_start_matches("FunctionCall(").trim_end_matches(')');
    let parts: Vec<&str> = split_top_level(content, ", ");
    
    if parts.len() >= 3 {
        let _file_info = parts[0]; // We're ignoring this for now
        let function = parse_ast_string(parts[1]);
        let args = parse_arguments_string(parts[2]);
        format!("{}({})", function, args)
    } else {
        log_debug(&format!("Unexpected FunctionCall format: {}", input));
        "unknownFunction()".to_string()
    }
}



/*
fn parse_function_call_string(input: &str) -> String {
        // Extract the member access part
        let member_access_re = Regex::new(r#"MemberAccess\(.*?, Variable\(Identifier \{ .*? name: "(\w+)" \}\), Identifier \{ .*? name: "(\w+)" \}\)"#).unwrap();
        let member_access = member_access_re.captures(input)
            .map(|cap| format!("{}.{}", &cap[1], &cap[2]))
            .unwrap_or_else(|| "unknownObject.unknownMethod000000".to_string());
    
        // Extract the arguments
        let args_re = Regex::new(r"\[(.*?)\]").unwrap();
        let args = args_re.captures(input)
            .and_then(|cap| Some(cap[1].to_string()))
            .unwrap_or_else(String::new);
    
        // Parse individual arguments
        let parsed_args: Vec<String> = args.split(", ")
            .map(|arg| {
                if arg.contains("Variable") {
                    let var_re = Regex::new(r#"name: "(\w+)""#).unwrap();
                    var_re.captures(arg)
                        .map(|cap| cap[1].to_string())
                        .unwrap_or_else(|| "unknownVar".to_string())
                } else if arg.contains("NumberLiteral") {
                    let num_re = Regex::new(r#"NumberLiteral\(.*?, "(\d+)", "(\d+)""#).unwrap();
                    num_re.captures(arg)
                        .map(|cap| format!("parseEther(\"{}\")", &cap[1]))
                        .unwrap_or_else(|| "parseEther(\"0\")".to_string())
                } else {
                    "unknownArg".to_string()
                }
            })
            .collect();
    
        // Combine everything
        format!("{}([{}]);", member_access, parsed_args.join(", "))
    }
    */
    /*
    fn parse_member_access_string(input: &str) -> String {
        log_debug(&format!("parse_member_access_string input: {}", input));

        let inner = input.trim_start_matches("MemberAccess(").trim_end_matches(')');
        let parts: Vec<&str> = split_top_level(inner, ", ");
        log_debug(&format!("parse_member_access_string parts: {:?}", parts));

        let result = if parts.len() >= 3 {
            let object = parse_ast_string(parts[1]);
            let member = parse_ast_string(parts[2]);
            format!("{}.{}", object, member)
        } else {
            "null".to_string()
        };
        log_debug(&format!("parse_member_access_string output: {}", result));

        result
    }

*/
/*

fn parse_member_access_string(input: &str) -> String {
    let parts: Vec<&str> = split_top_level(input, ", ");
    if parts.len() >= 3 {
        let object = parse_ast_string(parts[1]);
        let member = parse_ast_string(parts[2]);
        format!("{}.{}", object, member)
    } else {
        log_debug(&format!("Failed to parse member access: {}", input));

        "unknownObject.unknownMember2342342".to_string()
    }
}
*/

fn parse_member_access_string(input: &str) -> String {
    let content = input.trim_start_matches("MemberAccess(").trim_end_matches(')');
    let parts: Vec<&str> = split_top_level(content, ", ");
    
    if parts.len() >= 3 {
        let _file_info = parts[0]; // We're ignoring this for now
        let object = parse_ast_string(parts[1]);
        let member = parse_ast_string(parts[2]);
        format!("{}.{}", object, member)
    } else {
        "unknownObject.unknownMember".to_string()
    }
}


    
/*
    fn parse_variable_string(input: &str) -> String {
        log_debug(&format!("parse_variable_string input: {}", input));

        let name_start = input.rfind("name: \"").map(|i| i + 7).unwrap_or(0);
        let name_end = input[name_start..].find('"').map(|i| i + name_start).unwrap_or(input.len());
        let result = input[name_start..name_end].to_string();
        log_debug(&format!("parse_variable_string output: {:?}", result));

        result
    }


fn parse_variable_string(input: &str) -> String {
    
    if let Some(start) = input.rfind("name: \\\"") {
        let start = start + 8; // 7 for "name: \"" plus 1 for the extra backslash
        if let Some(end) = input[start..].find("\\\"") {
            return input[start..start+end].to_string();
        }
    }
    log_debug(&format!("Failed to parse variable: {}", input));

    "unknownVariable2222".to_string()
}
*/
fn parse_variable_string(input: &str) -> String {
    let content = input.trim_start_matches("Variable(").trim_end_matches(')');
    if let Some(identifier) = content.strip_prefix("Identifier ") {
        let parts: Vec<&str> = split_top_level(identifier.trim_matches('{').trim_matches('}'), ", ");
        for part in parts {
            if part.starts_with("name: ") {
                return part.trim_start_matches("name: ").trim_matches('"').to_string();
            }
        }
    }
    "unknownVariable".to_string()
}


/*
    fn parse_arguments_string(input: &str) -> String {
        log_debug(&format!("---------------START-----------------------------"));
        log_debug(&format!("parse_arguments_string input: {}", input));

        let args_str = input.trim_start_matches('[').trim_end_matches(']');
        let result = split_top_level(args_str, ", ")
            .into_iter()
            .map(parse_ast_string)
            .collect::<Vec<String>>()
            .join(", ");
        log_debug(&format!("parse_arguments_string result: {}", result));
        log_debug(&format!("-------------END-----------------------------------"));

        result
    }
*/
fn parse_arguments_string(input: &str) -> String {
    let args_str = input.trim_start_matches('[').trim_end_matches(']');
    let parsed_args = split_top_level(args_str, ", ")
        .into_iter()
        .map(parse_ast_string)
        .collect::<Vec<String>>();
    
    if parsed_args.is_empty() {
        log_debug(&format!("No arguments found in: {}", input));
    }
    
    parsed_args.join(", ")
}

    /*
    fn parse_number_literal_string(input: &str) -> String {
        log_debug(&format!("parse_number_literal_string input: {}", input));

        let parts: Vec<&str> = input.split(", ").collect();
        let result = if parts.len() >= 2 {
            let value = parts[1].trim_matches('"');
            format!("BigInt({}e18)", value)
        } else {
            "BigInt(0)".to_string()
        };
        log_debug(&format!("parse_number_literal_string output: {}", result));

        result
    }


fn parse_number_literal_string(input: &str) -> String {
    let parts: Vec<&str> = input.split(", ").collect();
    if parts.len() >= 3 {
        let value = parts[1].trim_matches('"');
        let decimals = parts[2].trim_matches('"').parse().unwrap_or(0);
        if decimals > 0 {
            format!("BigInt(\"{}{}\")", value, "0".repeat(decimals))
        } else {
            format!("BigInt({})", value)
        }
    } else {
        log_debug(&format!("Failed to parse number literal: {}", input));
        "BigInt(0)".to_string()
    }
}
*/
fn parse_number_literal_string(input: &str) -> String {
    let content = input.trim_start_matches("NumberLiteral(").trim_end_matches(')');
    let parts: Vec<&str> = split_top_level(content, ", ");
    if parts.len() >= 2 {
        parts[1].trim_matches('"').to_string()
    } else {
        "0".to_string()
    }
}


    /*
    fn split_top_level<'a>(input: &'a str, delimiter: &str) -> Vec<&'a str> {
        log_debug(&format!("split_top_level input: {}, delimiter: {}", input, delimiter));

        let mut result = Vec::new();
        let mut current_start = 0;
        let mut depth = 0;
    
        for (i, c) in input.char_indices() {
            match c {
                '(' | '[' => depth += 1,
                ')' | ']' => depth -= 1,
                _ if c == delimiter.chars().next().unwrap() && depth == 0 => {
                    if current_start < i {
                        result.push(&input[current_start..i]);
                    }
                    current_start = i + 1;
                },
                _ => {}
            }
        }
    
        if current_start < input.len() {
            result.push(&input[current_start..]);
        }
        log_debug(&format!("split_top_level input: {:?}", result));

        result
    }

*/
/*
FunctionCall(File(0, 459, 486), 
    MemberAccess(File(0, 459, 473), 
    Variable(
        Identifier { loc: File(0, 459, 464), name: \\\"token\\\" }), 
        Identifier { loc: File(0, 465, 473), name: \\\"transfer\\\" }), 
        [
        Variable(
            Identifier { loc: File(0, 474, 477), name: \\\"bob\\\" }), 
            NumberLiteral(File(0, 479, 485), \\\"100\\\", \\\"18\\\", None)])






fn split_top_level<'a>(input: &'a str, delimiter: &str) -> Vec<&'a str> {
    let mut result = Vec::new();
        let mut current_start = 0;
        let mut depth = 0;
        log_debug(&format!("delimiter here {}", delimiter));

        for (i, c) in input.char_indices() {
            log_debug(&format!("here is char {}", c));

            match c {
                '(' | '[' => {depth += 1;log_debug(&format!("we hit some depth up"));},
                ')' | ']' => {depth -= 1;log_debug(&format!("we hit some depth down"));},
                _ if c == delimiter.chars().next().unwrap() && depth == 0 => {

                    if current_start < i {
                        result.push(&input[current_start..i]);
                    }
                    current_start = i + delimiter.len();
                },
                _ => {}
            }
        }
    
        if current_start < input.len() {
            result.push(&input[current_start..]);
        }

        log_debug(&format!("split_top_level result: {:#?}", result));


        result
    }





fn split_top_level<'a>(input: &'a str, delimiter: &str) -> Vec<&'a str> {

        let mut result = Vec::new();
        let mut current_start = 0;
        let mut depth = 0;
        let mut in_quotes = false;
        log_debug(&format!("split_top_level input: {:#?}", input));
        log_debug(&format!("split_top_level delimiter: {}", delimiter));


        for (i, c) in input.char_indices() {
            match c {
                '(' => if !in_quotes { depth += 1 },
                ')' => if !in_quotes { 
                    log_debug(&format!("split_top_level push: {:#?}", &input[current_start..=i]));

                    depth -= 1;
                    if depth == 0 && input[current_start..=i].starts_with("FunctionCall(") {
                        result.push(&input[current_start..=i]);
                        current_start = i + 1;
                    }
                    log_debug(&format!("split_top_level current_result: {:#?}", result));

                },
                '"' => in_quotes = !in_quotes,
                '\\' => { /* Skip the next character if it's an escape */ },
                _ if c == delimiter.chars().next().unwrap() && depth == 0 && !in_quotes => {
                    if current_start < i {
                        result.push(&input[current_start..i]);
                    }
                    current_start = i + delimiter.len();
                },
                _ => {}
            }
        }
    
        if current_start < input.len() {
            result.push(&input[current_start..]);
        }
    
        // If no splits were made, return the entire input as a single item
        if result.is_empty() {
            result.push(input);
        }
        
        log_debug(&format!("split_top_level result: {:#?}", result));

        result
    }

*/

fn split_top_level<'a>(input: &'a str, delimiter: &str) -> Vec<&'a str> {
    let mut result = Vec::new();
    let mut current_start = 0;
    let mut depth = 0;
    let mut in_quotes = false;

    for (i, c) in input.char_indices() {
        match c {
            '(' => if !in_quotes { depth += 1 },
            ')' => if !in_quotes { depth -= 1 },
            '"' => in_quotes = !in_quotes,
            '\\' => { /* Skip the next character if it's an escape */ },
            _ if c == delimiter.chars().next().unwrap() && depth == 0 && !in_quotes => {
                result.push(input[current_start..i].trim());
                current_start = i + delimiter.len();
            },
            _ => {}
        }
    }

    if current_start < input.len() {
        result.push(input[current_start..].trim());
    }

    result
}

/*
    fn split_top_level<'a>(input: &'a str, delimiter: &str) -> Vec<&'a str> {
        let mut result = Vec::new();
        let mut current_start = 0;
        let mut depth = 0;
        let mut in_quotes = false;
        log_debug(&format!("split_top_level input: {:#?}", input));
        log_debug(&format!("split_top_level delimiter: {}", delimiter));
    
        for (i, c) in input.char_indices() {
            match c {
                '(' => {
                    if !in_quotes { 
                        depth += 1;
                        if depth == 1 && input[current_start..i].trim() == "FunctionCall" {
                            current_start = i + 1;
                        }
                    }
                },
                ')' => if !in_quotes { depth -= 1 },
                '"' => in_quotes = !in_quotes,
                '\\' => { /* Skip the next character if it's an escape */ },
                _ if c == delimiter.chars().next().unwrap() && depth == 1 && !in_quotes => {
                    result.push(input[current_start..i].trim());
                    current_start = i + delimiter.len();
                    log_debug(&format!("split_top_level push: {:#?}", result.last().unwrap()));
                },
                _ => {}
            }
        }
    
        if current_start < input.len() {
            result.push(input[current_start..].trim());
            log_debug(&format!("split_top_level push: {:#?}", result.last().unwrap()));
        }
    
        log_debug(&format!("split_top_level result: {:#?}", result));
    
        result
    }

*/

fn parse_ast(ast: &str) -> String {
    serde_json::from_str(ast)
        .map(|v| parse_value(&v))
        .unwrap_or_else(|_| "null".to_string())
}


fn parse_value(value: &Value) -> String {
    match value {
        Value::String(s) => parse_ast(s),
        Value::Object(map) => {
            if let Some(typ) = map.get("type") {
                match typ.as_str() {
                    Some("FunctionCall") => parse_function_call(map),
                    Some("MemberAccess") => parse_member_access(map),
                    Some("Variable") => parse_variable(map),
                    Some("Identifier") => parse_identifier(map),
                    Some("NumberLiteral") => parse_number_literal(map),
                    Some("HexNumberLiteral") => parse_hex_number_literal(map),
                    _ => value.to_string(),
                }
            } else {
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
    println!("argument: {:#?}",arg);
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



fn parse_hex_number_literal(map: &serde_json::Map<String, Value>) -> String {
    map.get("value")
       .and_then(|v| v.as_str())
       .map(|s| format!("BigInt(\"{}\")", s))
       .unwrap_or_else(|| "BigInt(0)".to_string())
}

fn extract_function_name(func_call: &str) -> String {
    // Look for the last occurrence of 'Identifier'
    log_debug(&format!("func_call input: {:?}", func_call));

    if let Some(last_identifier) = func_call.rfind("Identifier") {
        if let Some(start) = func_call[last_identifier..].find("name: \"") {
            let start = last_identifier + start + 7;
            if let Some(end) = func_call[start..].find('"') {
                return func_call[start..start+end].to_string();
            }
        }
    }
    "unknownFunction22".to_string()
}



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
    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let function = extract_function_name(param);

    log_debug(&format!("parse_assertion_function_helper param {:#?}:",param));
    log_debug(&format!("parse_assertion_function_helper function {:#?}:",function));

    out.write(&function)?;
    Ok(())
}

fn parse_assertion_args_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let args = extract_function_args(param);

    log_debug(&format!("parse_assertion_args_helper param {:#?}:",param));
    log_debug(&format!("parse_assertion_args_helper function {:#?}:",args));
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
