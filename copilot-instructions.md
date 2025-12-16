## Code Review Guidelines

When reviewing code in this repository, apply these standards strictly.
### Language Hierarchy

**Preferred:**
1. Zig — wherever C or C++ would be considered; low-level, comptime, no hidden control flow
2. Rust — systems, CLI, performance-critical
3. Ada/SPARK — safety-critical, formal verification
4. Haskell — pure functional, type-heavy domains
5. Elixir — concurrent, distributed, fault-tolerant systems
6. ReScript — frontend when JS interop needed
7. Chapel — parallel computing, HPC workloads
8. Julia — numerical computing, scientific applications

**Avoid:**
- C — use Zig instead
- C++ — use Zig or Rust instead
- Python — reject unless interfacing with Python-only libraries
- JavaScript — use ReScript or TypeScript instead
- Shell scripts over 50 lines — rewrite in a proper language

**Flag for justification:**
- Any use of Go, Java, C# without clear rationale
- C or C++ where Zig would suffice
---

### Error Handling

**Rust:**
- No `.unwrap()` or `.expect()` without a comment justifying why panic is acceptable
- Prefer `?` operator for propagation
- Use `thiserror` for library errors, `anyhow` for application errors
- No `panic!` in library code

**Haskell:**
- No `error` or `undefined` in production code
- Use `Either`, `Maybe`, or `ExceptT` for fallible operations
- Partial functions must be justified

**Elixir:**
- Use `{:ok, _}` / `{:error, _}` tuples consistently
- No bare `raise` without rescue strategy
- Supervisors must have explicit restart strategies

**Ada/SPARK:**
- All exceptions must be documented
- Prefer preconditions/postconditions over runtime checks
- SPARK contracts required for safety-critical sections

---

### Documentation

**Required:**
- All public functions/types must have doc comments
- Module-level documentation explaining purpose
- Examples for non-obvious APIs
- README must explain: what, why, how to build, how to use

**Format:**
- Use AsciiDoc (`.adoc`) for documentation files, not Markdown
- Exception: GitHub-required files (e.g., this file)

**Flag if missing:**
- CHANGELOG entries for user-facing changes
- Architecture decision records for significant design choices

---

### Tooling Preferences

**Flag violations:**

| Violation | Should Be |
|-----------|-----------|
| Docker | Podman |
| Makefile | justfile |
| GitHub Actions self-reference | GitLab CI preferred for personal projects |
| npm/yarn | pnpm (if JS unavoidable) |
| pip/poetry | Reject (avoid Python) |

**Build files:**
- `justfile` must be present for any project with build steps
- Recipes must have descriptions (`# comment above recipe`)

---

### Code Style

**General:**
- No commented-out code — delete it (git has history)
- No TODO without linked issue
- No magic numbers — use named constants
- Functions over 40 lines should be justified or split
- Cyclomatic complexity > 10 requires justification

**Naming:**
- Descriptive names over abbreviations
- No single-letter variables except: `i`, `j` for indices; `x`, `y` for coordinates; `f` for function parameters in HOFs
- British English spelling in user-facing strings and docs

**Formatting:**
- Must pass project's formatter (rustfmt, ormolu, mix format, etc.)
- No trailing whitespace
- Files must end with single newline
- UTF-8 encoding only

---

### Security

**Flag immediately:**
- Hardcoded credentials, API keys, secrets
- SQL string concatenation (injection risk)
- Unsanitised user input in shell commands
- Use of `eval` or equivalent in any language
- Disabled TLS verification
- Weak cryptographic choices (MD5, SHA1 for security)

**Require justification:**
- Any use of `unsafe` in Rust
- Any FFI calls
- Deserialisation of untrusted data
- Network requests to non-HTTPS endpoints

---

### Dependencies

**Flag for review:**
- Any new dependency — must justify why not stdlib
- Dependencies with < 100 GitHub stars or < 1 year old
- Dependencies without recent maintenance (> 1 year since release)
- Transitive dependency count increase > 10

**Reject:**
- Dependencies with known vulnerabilities
- Dependencies with incompatible licences (GPL in MIT project, etc.)
- Vendored code without licence attribution

---

### Testing

**Required:**
- Unit tests for all public functions with logic
- Integration tests for API boundaries
- Property-based tests for parsers, serialisers, algorithms

**Coverage:**
- New code should not decrease coverage percentage
- Critical paths require explicit test coverage

**Flag if missing:**
- Edge case tests (empty input, max values, unicode)
- Error path tests (not just happy path)

---

### Accessibility

**User interfaces must:**
- Support keyboard navigation
- Have sufficient colour contrast (WCAG AA minimum)
- Include alt text for images
- Not rely solely on colour to convey information
- Support screen readers where applicable

**CLI tools must:**
- Support `--help` and `--version`
- Use stderr for errors, stdout for output
- Return appropriate exit codes
- Support `NO_COLOR` environment variable

---

### Commits

**Reject PRs with:**
- Merge commits (rebase instead)
- WIP commits that should be squashed
- Commits mixing unrelated changes
- Commit messages without clear description

**Commit message format:**
