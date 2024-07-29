# Foundry2React

> [!WARNING]  
> This work is WIP, currently not all features are working. The output is currently for VITE, and some of the output is still not good. Also the tool name will change. You can see a sample output from [TokenTestComponent](./out/TokenTestComponent.tsx) 

## Description

Foundry2React is a powerful tool designed to automatically generate React components from Foundry Solidity test files. This project bridges the gap between smart contract testing and frontend development, allowing developers to quickly create interactive frontend representations of their Solidity tests.

## Features

- Parse Foundry Solidity test files
- Extract test functions, setup steps, and assertions
- Generate React components with viem and wagmi integration
- Simulate contract interactions and test scenarios in the browser
- Handlebars templating for flexible and customizable output

## Installation

### Prerequisites

- Rust (latest stable version)
- Node.js and npm (for running the generated React components)

### Steps

1. Clone the repository:

   ```
   git clone https://github.com/yourusername/foundry2react.git
   cd foundry2react
   ```

2. Build the project:

   ```
   cargo build --release
   ```

3. The binary will be available at `target/release/foundry2react`.

## Usage

### Basic Usage

```
foundry2react --test path/to/your/test.sol --output path/to/output/TestComponent.js
```

`foundry2react --test ../f2r_test/test/Token.t.sol --output ./out/generated.js --abi ../f2r_test/abi/token_abi.json`

`cargo run -- --test ../f2r_test/test/Token.t.sol --output ./out/generated.js --abi ../f2r_test/abi/token_abi.json`

### Options

- `--test` or `-t`: Path to the Foundry Solidity test file (required)
- `--output` or `-o`: Path for the output React component file (required)

## How It Works

1. **Parsing**: The tool parses the Solidity test file using the `solang-parser` library.
2. **Extraction**: It extracts relevant information such as state variables, setup functions, and test functions.
3. **Code Generation**: Using Handlebars templates, it generates a React component that simulates the test environment.
4. **ethers.js Integration**: The generated component uses ethers.js to interact with the Ethereum network and smart contracts.

## Generated Component Structure

The generated React component includes:

- State management for contract instance and test variables
- A setup function to initialize the contract and environment
- Individual functions for each test case
- UI elements to trigger setup and run tests
- Console output for test results and assertions

## Customization

You can customize the output by modifying the Handlebars template located at `templates/react_component.hbs`.

## Development

### Project Structure

- `src/main.rs`: Entry point of the application
- `src/foundry_test_parser.rs`: Solidity test file parser
- `src/generator.rs`: React component generator
- `templates/react_component.hbs`: Handlebars template for React component

### Adding New Features

1. To add support for new Solidity test constructs:

   - Update the `TestStep` enum in `foundry_test_parser.rs`
   - Modify the parsing logic in `extract_test_step` function

2. To change the React component output:
   - Modify the Handlebars template in `templates/react_component.hbs`
   - Update the `generate_js_code` function in `generator.rs` if necessary

### Running Tests

```
cargo test
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgements

- [Foundry](https://github.com/foundry-rs/foundry) - The inspiration for this project
- [solang-parser](https://github.com/hyperledger-labs/solang-parser) - For Solidity parsing capabilities
- [Handlebars-rust](https://github.com/sunng87/handlebars-rust) - For templating

## Contact

For any queries or suggestions, please open an issue on the GitHub repository.
