// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR PMPL-1.0-or-later

/**
 * Anvomidav VS Code Extension
 *
 * Provides language support for the Anvomidav figure skating DSL.
 */

const vscode = require('vscode');
const { LanguageClient, TransportKind } = require('vscode-languageclient/node');

let client;

/**
 * Activate the extension.
 * @param {vscode.ExtensionContext} context
 */
function activate(context) {
    const config = vscode.workspace.getConfiguration('anvomidav');
    const serverPath = config.get('lsp.path', 'anv-lsp');

    const serverOptions = {
        run: {
            command: serverPath,
            transport: TransportKind.stdio
        },
        debug: {
            command: serverPath,
            transport: TransportKind.stdio
        }
    };

    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'anvomidav' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.anv')
        }
    };

    client = new LanguageClient(
        'anvomidav',
        'Anvomidav Language Server',
        serverOptions,
        clientOptions
    );

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('anvomidav.restart', async () => {
            if (client) {
                await client.stop();
                await client.start();
                vscode.window.showInformationMessage('Anvomidav language server restarted');
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('anvomidav.showInfo', () => {
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'anvomidav') {
                vscode.window.showInformationMessage(
                    `Anvomidav file: ${editor.document.fileName}`
                );
            }
        })
    );

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
}

module.exports = { activate, deactivate };
