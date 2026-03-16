-- SPDX-FileCopyrightText: 2025 hyperpolymath
-- SPDX-License-Identifier: MIT OR PMPL-1.0-or-later

-- Anvomidav filetype plugin

vim.bo.commentstring = "// %s"
vim.bo.tabstop = 4
vim.bo.shiftwidth = 4
vim.bo.expandtab = true

-- Set up folding
vim.wo.foldmethod = "indent"
vim.wo.foldlevel = 99
