// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
// SPDX-License-Identifier: PMPL-1.0-or-later

import * as path from 'path';
import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    console.log('Anvomidav extension activated');

    // Start LSP client
    const serverOptions: ServerOptions = {
        command: getServerPath(),
        args: [],
        transport: TransportKind.stdio
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'anvomidav' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.anv')
        }
    };

    client = new LanguageClient(
        'anvomidavLanguageServer',
        'Anvomidav Language Server',
        serverOptions,
        clientOptions
    );

    client.start();

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('anvomidav.run', runProgram)
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('anvomidav.visualize', visualizeChoreography)
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('anvomidav.validate', validateISU)
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('anvomidav.debug', debugProgram)
    );
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

function getServerPath(): string {
    const config = vscode.workspace.getConfiguration('anvomidav');
    return config.get<string>('lspPath', 'anv-lsp');
}

function getDebuggerPath(): string {
    const config = vscode.workspace.getConfiguration('anvomidav');
    return config.get<string>('debuggerPath', 'anv-debug');
}

async function runProgram() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }

    const document = editor.document;
    if (document.languageId !== 'anvomidav') {
        vscode.window.showErrorMessage('Not an Anvomidav file');
        return;
    }

    await document.save();

    const terminal = vscode.window.createTerminal('Anvomidav');
    terminal.show();
    terminal.sendText(`anv run "${document.fileName}"`);
}

async function visualizeChoreography() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }

    const document = editor.document;
    if (document.languageId !== 'anvomidav') {
        vscode.window.showErrorMessage('Not an Anvomidav file');
        return;
    }

    await document.save();

    const outputPath = document.fileName.replace(/\.anv$/, '.svg');

    const terminal = vscode.window.createTerminal('Anvomidav Visualize');
    terminal.show();
    terminal.sendText(`anv visualize "${document.fileName}" -o "${outputPath}"`);

    vscode.window.showInformationMessage(`Visualization saved to ${outputPath}`);
}

async function validateISU() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }

    const document = editor.document;
    if (document.languageId !== 'anvomidav') {
        vscode.window.showErrorMessage('Not an Anvomidav file');
        return;
    }

    await document.save();

    const terminal = vscode.window.createTerminal('Anvomidav Validate');
    terminal.show();
    terminal.sendText(`anv validate "${document.fileName}"`);
}

async function debugProgram() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }

    const document = editor.document;
    if (document.languageId !== 'anvomidav') {
        vscode.window.showErrorMessage('Not an Anvomidav file');
        return;
    }

    await document.save();

    const debuggerPath = getDebuggerPath();
    const terminal = vscode.window.createTerminal('Anvomidav Debug');
    terminal.show();
    terminal.sendText(`${debuggerPath}`);
    terminal.sendText(`load "${document.fileName}"`);
}
