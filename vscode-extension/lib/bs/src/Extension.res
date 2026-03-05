// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
// SPDX-License-Identifier: PMPL-1.0-or-later

/**
 * Anvomidav VSCode Extension
 * Fully ported to ReScript v12
 */

module VsCode = {
  type disposable = {"dispose": unit => unit}

  type extensionContext = {
    subscriptions: array<disposable>,
    asAbsolutePath: string => string,
  }

  type textDocument = {
    languageId: string,
    fileName: string,
    save: unit => promise<unit>,
  }

  type textEditor = {document: textDocument}

  type terminal = {
    show: unit => unit,
    sendText: string => unit,
  }

  module Window = {
    @module("vscode") @scope("window")
    external showInformationMessage: string => promise<unit> = "showInformationMessage"

    @module("vscode") @scope("window")
    external showErrorMessage: string => promise<unit> = "showErrorMessage"

    @module("vscode") @scope("window")
    external createTerminal: string => terminal = "createTerminal"

    @module("vscode") @scope("window")
    @val
    external activeTextEditor: option<textEditor> = "activeTextEditor"
  }

  module Commands = {
    @module("vscode") @scope("commands")
    external registerCommand: (string, unit => promise<unit>) => disposable = "registerCommand"
  }

  module Workspace = {
    type configuration = {get: (string, string) => string}

    @module("vscode") @scope("workspace")
    external getConfiguration: string => configuration = "getConfiguration"

    @module("vscode") @scope("workspace")
    external createFileSystemWatcher: string => disposable = "createFileSystemWatcher"
  }
}

module Lsp = {
  type languageClient
  type serverOptions = {
    command: string,
    args: array<string>,
    transport: int,
  }
  type clientOptions = {
    documentSelector: array<{"scheme": string, "language": string}>,
    synchronize: {"fileEvents": VsCode.disposable},
  }

  @module("vscode-languageclient/node") @new
  external makeLanguageClient: (string, string, serverOptions, clientOptions) => languageClient =
    "LanguageClient"

  @send external start: languageClient => unit = "start"
  @send external stop: languageClient => promise<unit> = "stop"
}

let client: ref<option<Lsp.languageClient>> = ref(None)

let getServerPath = () => {
  let config = VsCode.Workspace.getConfiguration("anvomidav")
  config.get("lspPath", "anv-lsp")
}

let getDebuggerPath = () => {
  let config = VsCode.Workspace.getConfiguration("anvomidav")
  config.get("debuggerPath", "anv-debug")
}

let runProgram = async () => {
  switch VsCode.Window.activeTextEditor {
  | None => await VsCode.Window.showErrorMessage("No active editor")
  | Some(editor) => {
      let doc = editor.document
      if doc.languageId != "anvomidav" {
        await VsCode.Window.showErrorMessage("Not an Anvomidav file")
      } else {
        await doc.save()
        let terminal = VsCode.Window.createTerminal("Anvomidav")
        terminal.show()
        terminal.sendText(`anv run "${doc.fileName}"`)
      }
    }
  }
}

let visualizeChoreography = async () => {
  switch VsCode.Window.activeTextEditor {
  | None => await VsCode.Window.showErrorMessage("No active editor")
  | Some(editor) => {
      let doc = editor.document
      if doc.languageId != "anvomidav" {
        await VsCode.Window.showErrorMessage("Not an Anvomidav file")
      } else {
        await doc.save()
        let outputPath = String.replaceRegExp(doc.fileName, %re("/\.anv$/"), ".svg")
        let terminal = VsCode.Window.createTerminal("Anvomidav Visualize")
        terminal.show()
        terminal.sendText(`anv visualize "${doc.fileName}" -o "${outputPath}"`)
        await VsCode.Window.showInformationMessage(`Visualization saved to ${outputPath}`)
      }
    }
  }
}

let validateISU = async () => {
  switch VsCode.Window.activeTextEditor {
  | None => await VsCode.Window.showErrorMessage("No active editor")
  | Some(editor) => {
      let doc = editor.document
      if doc.languageId != "anvomidav" {
        await VsCode.Window.showErrorMessage("Not an Anvomidav file")
      } else {
        await doc.save()
        let terminal = VsCode.Window.createTerminal("Anvomidav Validate")
        terminal.show()
        terminal.sendText(`anv validate "${doc.fileName}"`)
      }
    }
  }
}

let debugProgram = async () => {
  switch VsCode.Window.activeTextEditor {
  | None => await VsCode.Window.showErrorMessage("No active editor")
  | Some(editor) => {
      let doc = editor.document
      if doc.languageId != "anvomidav" {
        await VsCode.Window.showErrorMessage("Not an Anvomidav file")
      } else {
        await doc.save()
        let debuggerPath = getDebuggerPath()
        let terminal = VsCode.Window.createTerminal("Anvomidav Debug")
        terminal.show()
        terminal.sendText(`${debuggerPath}`)
        terminal.sendText(`load "${doc.fileName}"`)
      }
    }
  }
}

let activate = (context: VsCode.extensionContext) => {
  Js.log("Anvomidav extension activated")

  let serverOptions: Lsp.serverOptions = {
    command: getServerPath(),
    args: [],
    transport: 1, // TransportKind.stdio
  }

  let clientOptions: Lsp.clientOptions = {
    documentSelector: [{"scheme": "file", "language": "anvomidav"}],
    synchronize: {
      "fileEvents": VsCode.Workspace.createFileSystemWatcher("**/*.anv"),
    },
  }

  let c = Lsp.makeLanguageClient(
    "anvomidavLanguageServer",
    "Anvomidav Language Server",
    serverOptions,
    clientOptions,
  )

  client := Some(c)
  Lsp.start(c)

  // Register commands
  let _ = Js.Array2.push(
    context.subscriptions,
    VsCode.Commands.registerCommand("anvomidav.run", runProgram),
  )
  let _ = Js.Array2.push(
    context.subscriptions,
    VsCode.Commands.registerCommand("anvomidav.visualize", visualizeChoreography),
  )
  let _ = Js.Array2.push(
    context.subscriptions,
    VsCode.Commands.registerCommand("anvomidav.validate", validateISU),
  )
  let _ = Js.Array2.push(
    context.subscriptions,
    VsCode.Commands.registerCommand("anvomidav.debug", debugProgram),
  )
}

let deactivate = () => {
  switch client.contents {
  | None => Promise.resolve()
  | Some(c) => Lsp.stop(c)
  }
}
