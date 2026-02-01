// SPDX-FileCopyrightText: 2025 hyperpolymath
// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
// Anvomidav VSCode Extension - ReScript Implementation

open VSCode

let client: ref<option<LanguageClient.t>> = ref(None)

let startLanguageClient = (context: extensionContext) => {
  let config = Workspace.getConfiguration("anvomidav")
  let serverPath = switch Workspace.get(config, "lsp.path") {
  | Some(path) => path
  | None => "anv-lsp"
  }

  let serverOptions: LanguageClient.serverOptions = {
    run: {
      command: serverPath,
      transport: Stdio,
    },
    debug: {
      command: serverPath,
      transport: Stdio,
    },
  }

  let clientOptions: LanguageClient.clientOptions = {
    documentSelector: [{scheme: "file", language: "anvomidav"}],
    synchronize: {
      fileEvents: Workspace.createFileSystemWatcher("**/*.anv"),
    },
  }

  let languageClient = LanguageClient.make(
    "anvomidav",
    "Anvomidav Language Server",
    serverOptions,
    clientOptions,
  )

  client := Some(languageClient)
  let _ = LanguageClient.start(languageClient)
  ()
}

let restartServer = (context: extensionContext) => {
  switch client.contents {
  | Some(c) =>
    let _ = LanguageClient.stop(c)
    startLanguageClient(context)
  | None => startLanguageClient(context)
  }
}

let showInfo = () => {
  let _ = Window.showInformationMessage("Anvomidav LSP extension")
  ()
}

let activate = (context: extensionContext) => {
  startLanguageClient(context)

  // Register commands
  let restartCmd = Commands.registerCommand("anvomidav.restart", () =>
    Js.Promise.make((~resolve, ~reject as _) => {
      restartServer(context)
      resolve(.)
    })
  )
  let infoCmd = Commands.registerCommand("anvomidav.showInfo", () =>
    Js.Promise.make((~resolve, ~reject as _) => {
      showInfo()
      resolve(.)
    })
  )

  let _ = Js.Array2.push(context.subscriptions, restartCmd)
  let _ = Js.Array2.push(context.subscriptions, infoCmd)
  ()
}

let deactivate = () => {
  switch client.contents {
  | Some(c) => Some(LanguageClient.stop(c))
  | None => None
  }
}
