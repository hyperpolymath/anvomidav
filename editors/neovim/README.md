# Anvomidav for Neovim

This directory contains configuration for Anvomidav language support in Neovim.

## Installation

### Using lazy.nvim

```lua
{
    "hyperpolymath/anvomidav",
    config = function()
        require("anvomidav").setup()
    end,
    ft = "anvomidav",
}
```

### Manual Setup

1. Copy the files from this directory to your Neovim config
2. Add the following to your init.lua:

```lua
-- Add filetype detection
vim.filetype.add({
    extension = {
        anv = "anvomidav",
    },
})

-- Configure LSP
local lspconfig = require("lspconfig")
local configs = require("lspconfig.configs")

if not configs.anvomidav then
    configs.anvomidav = {
        default_config = {
            cmd = { "anv-lsp" },
            filetypes = { "anvomidav" },
            root_dir = lspconfig.util.root_pattern(".git", "*.anv"),
            settings = {},
        },
    }
end

lspconfig.anvomidav.setup({
    on_attach = function(client, bufnr)
        -- Your on_attach function
    end,
})
```

### Tree-sitter

To enable tree-sitter highlighting, add the parser:

```lua
local parser_config = require("nvim-treesitter.parsers").get_parser_configs()
parser_config.anvomidav = {
    install_info = {
        url = "https://github.com/hyperpolymath/anvomidav",
        files = { "tree-sitter-anvomidav/src/parser.c" },
        branch = "main",
    },
    filetype = "anvomidav",
}
```

Then run `:TSInstall anvomidav`
