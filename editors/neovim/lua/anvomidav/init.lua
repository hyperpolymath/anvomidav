-- SPDX-FileCopyrightText: 2025 hyperpolymath
-- SPDX-License-Identifier: MIT OR PMPL-1.0-or-later

-- Anvomidav Neovim plugin

local M = {}

M.config = {
    lsp_path = "anv-lsp",
    auto_start = true,
    discipline = "singles",
}

function M.setup(opts)
    opts = opts or {}
    M.config = vim.tbl_deep_extend("force", M.config, opts)

    -- Register filetype
    vim.filetype.add({
        extension = {
            anv = "anvomidav",
        },
    })

    -- Setup LSP if lspconfig is available
    local ok, lspconfig = pcall(require, "lspconfig")
    if ok then
        local configs = require("lspconfig.configs")

        if not configs.anvomidav then
            configs.anvomidav = {
                default_config = {
                    cmd = { M.config.lsp_path },
                    filetypes = { "anvomidav" },
                    root_dir = lspconfig.util.root_pattern(".git", "*.anv"),
                    settings = {
                        anvomidav = {
                            discipline = M.config.discipline,
                        },
                    },
                },
            }
        end

        if M.config.auto_start then
            lspconfig.anvomidav.setup({})
        end
    end

    -- Setup treesitter if available
    local ts_ok, parsers = pcall(require, "nvim-treesitter.parsers")
    if ts_ok then
        local parser_config = parsers.get_parser_configs()
        parser_config.anvomidav = {
            install_info = {
                url = "https://github.com/hyperpolymath/anvomidav",
                files = { "tree-sitter-anvomidav/src/parser.c" },
                branch = "main",
            },
            filetype = "anvomidav",
        }
    end
end

-- Commands
function M.restart_lsp()
    vim.cmd("LspRestart anvomidav")
end

function M.show_info()
    local bufnr = vim.api.nvim_get_current_buf()
    local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
    local source = table.concat(lines, "\n")

    -- Count elements
    local jumps = 0
    local spins = 0
    local steps = 0

    for line in source:gmatch("[^\n]+") do
        if line:match("^%s*jump") then jumps = jumps + 1 end
        if line:match("^%s*spin") then spins = spins + 1 end
        if line:match("^%s*step") then steps = steps + 1 end
    end

    vim.notify(string.format(
        "Anvomidav Program Info:\n  Jumps: %d\n  Spins: %d\n  Steps: %d",
        jumps, spins, steps
    ), vim.log.levels.INFO)
end

-- Create user commands
vim.api.nvim_create_user_command("AnvRestartLsp", M.restart_lsp, {})
vim.api.nvim_create_user_command("AnvInfo", M.show_info, {})

return M
