// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later

/**
 * Anvomidav VS Code Extension
 *
 * Provides language support for the Anvomidav figure skating DSL.
 */

import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

/**
 * Activate the extension.
 */
export function activate(context: vscode.ExtensionContext): void {
    const config = vscode.workspace.getConfiguration('anvomidav');
    const serverPath = config.get<string>('lsp.path', 'anv-lsp');

    const serverOptions: ServerOptions = {
        run: {
            command: serverPath,
            transport: TransportKind.stdio
        },
        debug: {
            command: serverPath,
            transport: TransportKind.stdio
        }
    };

    const clientOptions: LanguageClientOptions = {
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
export function deactivate(): Thenable<void> | undefined {
    if (client) {
        return client.stop();
    }
    return undefined;
}
