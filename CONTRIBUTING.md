# Contributing to Anvomidav

Thank you for your interest in contributing to Anvomidav, the domain-specific language for figure skating choreography.

## Language Policy

This project follows the Hyperpolymath Standard for language selection:

| Allowed | Use Case |
|---------|----------|
| **Rust** | Core implementation, CLI, libraries |
| **ReScript** | Future web UI components |
| **Deno** | JavaScript runtime (if needed) |

**Not Permitted:** TypeScript, Node.js, Go, Python (except SaltStack).

## Development Setup

### Prerequisites

- Rust 1.75+ (stable)
- Cargo

### Building

```bash
# Clone the repository
git clone https://github.com/hyperpolymath/anvomidav.git
cd anvomidav

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run clippy
cargo clippy --workspace
```

### Project Structure

```
anvomidav/
├── crates/
│   ├── anv-core/       # Core types and utilities
│   ├── anv-syntax/     # Lexer and parser
│   ├── anv-types/      # Type checking
│   ├── anv-semantics/  # ISU rule validation
│   ├── anv-ir/         # Intermediate representation
│   ├── anv-viz/        # SVG visualization
│   └── anv-cli/        # Command-line interface
├── examples/           # Example .anv programs
└── docs/               # Documentation
```

## Making Changes

### Code Style

- Follow Rust conventions (run `cargo fmt`)
- No warnings allowed (run `cargo clippy`)
- All public APIs must be documented
- SPDX license headers on all source files:
  ```rust
  // SPDX-FileCopyrightText: 2025 hyperpolymath
  // SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
  ```

### Testing

- Write tests for new functionality
- Ensure all tests pass: `cargo test --workspace`
- Add integration tests for CLI changes

### Commit Messages

Use clear, descriptive commit messages:

```
Add pairs skating element validation

- Implement lift group validation
- Add throw jump ISU limits
- Update tests for pairs short program
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Run tests and clippy
5. Commit with clear messages
6. Push to your fork
7. Open a Pull Request

### PR Checklist

- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation updated (if applicable)
- [ ] CHANGELOG updated (for user-facing changes)

## Areas for Contribution

### Good First Issues

- Add more example programs for different disciplines
- Improve error messages
- Add test cases for edge cases

### Intermediate

- Implement additional choreographic elements
- Enhance SVG visualizations
- Add more ISU rule validations

### Advanced

- Tree-sitter grammar for editor support
- LSP server implementation
- Animation/timeline export

## Getting Help

- Open an issue for questions
- Check existing issues before creating new ones
- Use discussions for general questions

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT OR AGPL-3.0-or-later).
