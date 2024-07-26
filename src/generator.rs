use handlebars::Handlebars;
use serde_json::json;
use std::fs;
use std::path::Path;
use eyre::Result;


//mod foundry_test_parser;
use crate::foundry_test_parser::TestContract;


pub fn generate_js_code(test_contract: &TestContract) -> Result<String, handlebars::RenderError> {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("component", include_str!("../templates/react_component.hbs"))?;
    
    // Helper to capitalize the first letter of a string
    handlebars.register_helper(
        "capitalize",
        Box::new(|h: &handlebars::Helper,
                  _: &handlebars::Handlebars,
                  _: &handlebars::Context,
                  _: &mut handlebars::RenderContext,
                  out: &mut dyn handlebars::Output|
         -> handlebars::HelperResult {
            let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
            let capitalized = param.chars().next().map(|c| c.to_uppercase().collect::<String>() + &param[1..]).unwrap_or_default();
            out.write(&capitalized)?;
            Ok(())
        }),
    );

    // Helper to parse arguments
    handlebars.register_helper(
        "parseArg",
        Box::new(|h: &handlebars::Helper,
                  _: &handlebars::Handlebars,
                  _: &handlebars::Context,
                  _: &mut handlebars::RenderContext,
                  out: &mut dyn handlebars::Output|
         -> handlebars::HelperResult {
            let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
            let parsed = parse_argument(param);
            out.write(&parsed)?;
            Ok(())
        }),
    );

    // Helper to parse assertions
    handlebars.register_helper(
        "parseAssertion",
        Box::new(|h: &handlebars::Helper,
                  _: &handlebars::Handlebars,
                  _: &handlebars::Context,
                  _: &mut handlebars::RenderContext,
                  out: &mut dyn handlebars::Output|
         -> handlebars::HelperResult {
            let params: Vec<_> = h.params().iter().filter_map(|v| v.value().as_str()).collect();
            let parsed = parse_assertion(&params);
            out.write(&parsed)?;
            Ok(())
        }),
    );

    let data = json!({
        "contractName": test_contract.name,
        "stateVariables": test_contract.state_variables,
        "setupFunction": test_contract.setup,
        "testFunctions": test_contract.test_functions
    });

    handlebars.render("component", &data)
}

fn parse_argument(arg: &str) -> String {
    if arg.starts_with("Variable(") {
        // Extract variable name
        let start = arg.find("name: \"").map(|i| i + 7).unwrap_or(0);
        let end = arg[start..].find('"').map(|i| i + start).unwrap_or(arg.len());
        arg[start..end].to_string()
    } else if arg.starts_with("NumberLiteral(") {
        // Extract number value
        let start = arg.find(", \"").map(|i| i + 3).unwrap_or(0);
        let end = arg[start..].find('"').map(|i| i + start).unwrap_or(arg.len());
        format!("ethers.utils.parseUnits(\"{}\")", &arg[start..end])
    } else {
        arg.to_string()
    }
}

fn parse_assertion(args: &[&str]) -> String {
    if args.len() == 2 {
        format!("{} === {}", parse_argument(args[0]), parse_argument(args[1]))
    } else {
        args.iter().map(|&arg| parse_argument(arg)).collect::<Vec<_>>().join(" ")
    }
}