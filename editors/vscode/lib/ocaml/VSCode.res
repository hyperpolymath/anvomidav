// SPDX-License-Identifier: MIT OR Apache-2.0
// VSCode API bindings for ReScript

type disposable
type fileSystemWatcher

type extensionContext = {
  subscriptions: array<disposable>,
}

module Workspace = {
  type configuration

  @module("vscode") @scope("workspace")
  external getConfiguration: string => configuration = "getConfiguration"

  @send external get: (configuration, string) => option<string> = "get"

  @module("vscode") @scope("workspace")
  external createFileSystemWatcher: string => fileSystemWatcher = "createFileSystemWatcher"
}

module Window = {
  @module("vscode") @scope("window")
  external showInformationMessage: string => promise<option<string>> = "showInformationMessage"

  @module("vscode") @scope("window")
  external showErrorMessage: string => promise<option<string>> = "showErrorMessage"
}

module Commands = {
  @module("vscode") @scope("commands")
  external registerCommand: (string, unit => promise<unit>) => disposable = "registerCommand"
}

module LanguageClient = {
  type transportKind = | @as(0) Stdio

  type optionsType = {env?: Js.Dict.t<string>}

  type runOptions = {
    command: string,
    transport: transportKind,
    options?: optionsType,
  }

  type serverOptions = {
    run: runOptions,
    debug: runOptions,
  }

  type documentFilter = {
    scheme: string,
    language: string,
  }

  type synchronizeOptions = {fileEvents: fileSystemWatcher}

  type clientOptions = {
    documentSelector: array<documentFilter>,
    synchronize: synchronizeOptions,
  }

  type t

  @module("vscode-languageclient/node") @new
  external make: (string, string, serverOptions, clientOptions) => t = "LanguageClient"

  @send external start: t => promise<unit> = "start"
  @send external stop: t => promise<unit> = "stop"
}
