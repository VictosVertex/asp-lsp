import { workspace, type ExtensionContext, window } from "vscode";
import * as path from "path";
import {
  type Executable,
  LanguageClient,
  type LanguageClientOptions,
  type ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient;

function getServerExecutable(context: ExtensionContext): string {
  switch (process.platform) {
    case "win32":
      return context.asAbsolutePath(
        path.join("server", "windows", "asp-lsp.exe")
      );
    case "darwin":
      return context.asAbsolutePath(path.join("server", "macos", "asp-lsp"));
    case "linux":
      return context.asAbsolutePath(path.join("server", "linux", "asp-lsp"));
    default:
      throw new Error(`Unsupported platform: ${process.platform}`);
  }
}

export async function activate(context: ExtensionContext): Promise<void> {
  const traceOutputChannel = window.createOutputChannel("asp-lsp trace");

  const command = process.env.SERVER_PATH ?? getServerExecutable(context);

  const run: Executable = {
    command,
    options: {
      env: {
        ...process.env,
        RUST_LOG: "debug",
      },
    },
  };

  const serverOptions: ServerOptions = {
    run,
    debug: run,
  };

  // Options to control the language client
  const clientOptions: LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [{ scheme: "file", language: "asp" }],
    traceOutputChannel,
  };

  // Create the language client and start the client.
  client = new LanguageClient(
    "asp-lsp",
    "ASP language server",
    serverOptions,
    clientOptions
  );

  console.log("Running asp-lsp extention");
  await client.start();
}

export function deactivate(): Thenable<void> | undefined {
  console.log("Exiting asp-lsp extention");
  if (client === null) {
    return undefined;
  }
  return client.stop();
}
