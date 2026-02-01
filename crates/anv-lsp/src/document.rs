// SPDX-License-Identifier: MIT
// Document state management for Anvomidav LSP

use ropey::Rope;

pub struct DocumentState {
    pub content: Rope,
    pub version: i32,
}

impl DocumentState {
    pub fn new(content: Rope, version: i32) -> Self {
        Self { content, version }
    }

    pub fn source(&self) -> String {
        self.content.to_string()
    }
}
