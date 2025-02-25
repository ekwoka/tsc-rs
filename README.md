# tsc-rs

A TypeScript type checker written in Rust, aiming to provide fast and accurate type checking capabilities. This project is an experimental alternative to the official TypeScript compiler's type checker, with a focus on performance.

## About

This project is currently being developed primarily through AI pair programming using [Windsurf](https://www.codeium.com/windsurf), an agentic IDE that enables collaborative development with AI agents. The codebase is being crafted through interactions with AI assistants, making this an interesting experiment in AI-driven development of a complex type system.

## Current Status

This is an early-stage project that implements core type checking functionality. The type checker is being developed incrementally, with a focus on correctness and test coverage.

### Feature Support

- Basic Types:
  - [x] `number`
  - [x] `string`
  - [x] `boolean`
  - [x] `null`
  - [x] `undefined`
  - [x] `bigint`
  - [x] `symbol`
  - [x] `void`
  - [x] `any`
  - [x] `unknown`
  - [x] `never`
  - [x] `object`

- Literal Types:
  - [x] String literals
  - [x] Number literals
  - [x] Boolean literals

- Compound Types:
  - [x] Union types
  - [x] Array types
  - [x] Tuple types
  - [x] Function types (with parameter and return type checking)

- Type Checking Features:
  - [x] Variable declarations with type annotations
  - [x] Function declarations with parameter and return type checking
  - [x] Type inference for variable initialization
  - [x] Type compatibility checking
  - [x] Basic error reporting

- Interfaces and Classes
  - [ ] Interface declarations
  - [ ] Class declarations with inheritance
  - [ ] Implementation of interfaces
  - [ ] Access modifiers

- Advanced Types
  - [ ] Intersection types
  - [ ] Generic types
  - [ ] Mapped types
  - [ ] Conditional types
  - [ ] Index types
  - [ ] Utility types (Pick, Omit, etc.)

- Type System Features
  - [ ] Type narrowing
  - [ ] Type guards
  - [ ] Type assertions
  - [ ] Optional properties
  - [ ] Readonly properties
  - [ ] Method signatures

- Module System
  - [ ] Import/export declarations
  - [ ] Namespace support
  - [ ] Module resolution

- Enhanced Error Handling
  - [ ] Detailed error messages
  - [ ] Source code location in errors
  - [ ] Suggestions for fixes

## Development

The project follows these development principles:
- Modules are kept small and focused
- Changes to existing code are limited when adding new functionality
- New features are developed test-first
- Code quality is maintained through continuous testing (`cargo test`)
- Type checking validity is verified with `cargo check`

## Contributing

This project is an experiment in AI-assisted development, but human contributions are welcome! If you're interested in contributing, please feel free to open issues or submit pull requests.
