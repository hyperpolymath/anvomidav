"use strict";
// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR PMPL-1.0-or-later
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
/**
 * Anvomidav VS Code Extension
 *
 * Provides language support for the Anvomidav figure skating DSL.
 */
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
let client;
/**
 * Activate the extension.
 */
function activate(context) {
    const config = vscode.workspace.getConfiguration('anvomidav');
    const serverPath = config.get('lsp.path', 'anv-lsp');
    const serverOptions = {
        run: {
            command: serverPath,
            transport: node_1.TransportKind.stdio
        },
        debug: {
            command: serverPath,
            transport: node_1.TransportKind.stdio
        }
    };
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'anvomidav' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.anv')
        }
    };
    client = new node_1.LanguageClient('anvomidav', 'Anvomidav Language Server', serverOptions, clientOptions);
    // Register commands
    context.subscriptions.push(vscode.commands.registerCommand('anvomidav.restart', async () => {
        if (client) {
            await client.stop();
            await client.start();
            vscode.window.showInformationMessage('Anvomidav language server restarted');
        }
    }));
    context.subscriptions.push(vscode.commands.registerCommand('anvomidav.showInfo', () => {
        const editor = vscode.window.activeTextEditor;
        if (editor && editor.document.languageId === 'anvomidav') {
            vscode.window.showInformationMessage(`Anvomidav file: ${editor.document.fileName}`);
        }
    }));
    // Start the client
    client.start();
}
/**
 * Deactivate the extension.
 */
function deactivate() {
    if (client) {
        return client.stop();
    }
    return undefined;
}
//# sourceMappingURL=extension.js.map