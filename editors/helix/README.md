# Anvomidav for Helix

This directory contains configuration for Anvomidav language support in the Helix editor.

## Installation

### 1. Add Language Configuration

Add the following to your `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "anvomidav"
scope = "source.anv"
file-types = ["anv"]
comment-token = "//"
indent = { tab-width = 4, unit = "    " }
language-servers = ["anvomidav-lsp"]

[language-server.anvomidav-lsp]
command = "anv-lsp"
```

### 2. Add Tree-sitter Grammar

The tree-sitter grammar needs to be built and installed.

```bash
# Clone the repository
git clone https://github.com/hyperpolymath/anvomidav
cd anvomidav/tree-sitter-anvomidav

# Build the grammar (requires tree-sitter CLI)
tree-sitter generate

# Copy to Helix runtime
mkdir -p ~/.config/helix/runtime/grammars/sources/anvomidav
cp -r src ~/.config/helix/runtime/grammars/sources/anvomidav/

# Copy queries
mkdir -p ~/.config/helix/runtime/queries/anvomidav
cp queries/*.scm ~/.config/helix/runtime/queries/anvomidav/
```

### 3. Build the Grammar

```bash
cd ~/.config/helix
hx --grammar build
```

## Verification

Open a `.anv` file in Helix. You should see:
- Syntax highlighting
- LSP diagnostics
- Hover information
- Completions
