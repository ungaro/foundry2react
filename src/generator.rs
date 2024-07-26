use handlebars::Handlebars;
use serde_json::json;
use std::fs;
use std::path::Path;
use eyre::Result;


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
    let function = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    
    let parsed_args: Vec<String> = h.params().iter()
        .skip(1)  // Skip the first parameter (function name)
        .filter_map(|param| param.value().as_str())
        .map(parse_argument)
        .collect();

    let function_call = format!("contract.read.{}([{}])", function, parsed_args.join(", "));
    out.write(&function_call)?;
    Ok(())
}


fn parse_arg_helper(h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let parsed = parse_argument(param);
    out.write(&parsed)?;
    Ok(())
}

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

fn parse_argument(arg: &str) -> String {
    if arg.starts_with("Variable(") {
        let start = arg.find("name: \"").map(|i| i + 7).unwrap_or(0);
        let end = arg[start..].find('"').map(|i| i + start).unwrap_or(arg.len());
        arg[start..end].to_string()
    } else if arg.starts_with("NumberLiteral(") {
        let start = arg.find(", \"").map(|i| i + 3).unwrap_or(0);
        let end = arg[start..].find('"').map(|i| i + start).unwrap_or(arg.len());
        format!("BigInt({}e18)", &arg[start..end])
    } else {
        arg.to_string()
    }
}

fn extract_function_name(arg: &str) -> String {
    if let Some(start) = arg.find("name: \"") {
        let start = start + 7;
        if let Some(end) = arg[start..].find('"') {
            return arg[start..start+end].to_string();
        }
    }
    "unknownFunction".to_string()
}

fn extract_function_args(arg: &str) -> Vec<String> {
    if let Some(start) = arg.find('[') {
        if let Some(end) = arg[start..].rfind(']') {
            return arg[start+1..start+end]
                .split(',')
                .map(|s| parse_argument(s.trim()))
                .collect();
        }
    }
    vec![]
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
