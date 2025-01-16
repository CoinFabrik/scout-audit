import * as vscode from "vscode";
import commandExists from "command-exists";
import fs from "fs/promises";
import path from "path";
import { spawn } from "child_process";

const CARGO_TOML = "Cargo.toml";

interface CompilerMessage {
  reason: string;
  message: {
    rendered: string;
    message: string;
    level: string;
    code?: {
      code: string;
    };
    spans: Array<{
      file_name: string;
      line_start: number;
      line_end: number;
      column_start: number;
      column_end: number;
      text: Array<{
        text: string;
      }>;
    }>;
  };
}

interface CargoProject {
  root: string;
}

let diagnosticCollection: vscode.DiagnosticCollection;
let outputChannel: vscode.OutputChannel;

export async function activate(context: vscode.ExtensionContext) {
  try {
    outputChannel = vscode.window.createOutputChannel("Scout Audit");
    context.subscriptions.push(outputChannel);

    outputChannel.appendLine("Scout Audit extension activated");

    diagnosticCollection =
      vscode.languages.createDiagnosticCollection("scout-audit");
    context.subscriptions.push(diagnosticCollection);

    if (!(await isWorkspaceValid())) {
      outputChannel.appendLine("Invalid workspace: Cargo.toml not found");
      await vscode.window.showErrorMessage(
        "Invalid workspace: Cargo.toml not found."
      );
      return;
    }

    if (!(await checkAndInstallScout())) return;

    const runScoutCommand = vscode.commands.registerCommand(
      "scout-audit.run",
      () => {
        outputChannel.appendLine("Manual Scout audit triggered");
        return runScout();
      }
    );
    context.subscriptions.push(runScoutCommand);

    context.subscriptions.push(
      vscode.workspace.onDidSaveTextDocument((document) => {
        if (document.languageId === "rust") {
          outputChannel.appendLine(`File saved: ${document.uri.fsPath}`);
          void runScout();
        }
      })
    );

    outputChannel.show(true);
  } catch (error) {
    const errorMessage = `Activation error: ${
      error instanceof Error ? error.message : String(error)
    }`;
    outputChannel.appendLine(errorMessage);
    await vscode.window.showErrorMessage(errorMessage);
  }
}

function getProjectWorkspaceRoot(): string | undefined {
  const workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders) return undefined;

  const cargoTomlPath = path.join(workspaceFolders[0].uri.fsPath, CARGO_TOML);
  const workspaceRoot = path.dirname(cargoTomlPath);
  return workspaceRoot;
}

async function getCargoWorkspaceRoot(
  currentFilePath: string
): Promise<string | undefined> {
  return new Promise((resolve) => {
    const cargo = spawn("cargo", ["locate-project", "--workspace"], {
      cwd: currentFilePath,
    });

    let stdout = "";
    cargo.stdout.on("data", (data: Buffer) => {
      stdout += data.toString();
    });

    cargo.on("close", (code) => {
      if (code !== 0) {
        outputChannel.appendLine(
          `Cargo locate-project failed with code ${code ?? "unknown"}`
        );
        resolve(undefined);
        return;
      }

      try {
        const project = JSON.parse(stdout.trim()) as CargoProject;
        const workspaceRoot = path.dirname(project.root);
        resolve(workspaceRoot);
      } catch (e) {
        outputChannel.appendLine(
          `Error parsing cargo locate-project output: ${
            e instanceof Error ? e.message : String(e)
          }`
        );
        resolve(undefined);
      }
    });
  });
}

async function runScout() {
  if (!vscode.window.activeTextEditor) {
    outputChannel.appendLine("No active text editor");
    return;
  }

  const projectWorkspaceRoot = getProjectWorkspaceRoot()!;
  outputChannel.appendLine(`Project root: ${projectWorkspaceRoot}`);

  const workspaceRoot = await getCargoWorkspaceRoot(projectWorkspaceRoot);
  if (!workspaceRoot) {
    outputChannel.appendLine("Could not determine cargo workspace root");
    return;
  }
  outputChannel.appendLine(`Workspace root: ${workspaceRoot}`);

  return new Promise<void>((resolve) => {
    const scout = spawn(
      "cargo",
      ["scout-audit", "--", "--message-format=json"],
      {
        cwd: projectWorkspaceRoot,
        env: { ...process.env, RUST_BACKTRACE: "0" },
      }
    );

    let output = "";
    scout.stdout.on("data", (data: Buffer) => {
      output += data.toString();
    });

    scout.on("close", () => {
      try {
        const diagnosticMap = new Map<string, vscode.Diagnostic[]>();

        output.split("\n").forEach((line) => {
          if (!line.trim()) return;

          try {
            const message = JSON.parse(line) as CompilerMessage;

            if (message.reason !== "compiler-message" || !message.message)
              return;

            if (!message.message.spans || message.message.spans.length === 0) {
              return;
            }

            message.message.spans.forEach((span) => {
              if (!span.file_name) {
                return;
              }

              const targetFileName = path.join(workspaceRoot, span.file_name);

              const range = new vscode.Range(
                span.line_start - 1,
                span.column_start - 1,
                span.line_end - 1,
                span.column_end - 1
              );

              const severity =
                message.message.level === "error"
                  ? vscode.DiagnosticSeverity.Error
                  : vscode.DiagnosticSeverity.Warning;

              const vsDiagnostic = new vscode.Diagnostic(
                range,
                message.message.rendered || message.message.message,
                severity
              );

              if (message.message.code) {
                vsDiagnostic.code = String(message.message.code.code);
              }
              vsDiagnostic.source = "Scout";

              const diagnostics = diagnosticMap.get(targetFileName) || [];
              diagnostics.push(vsDiagnostic);
              diagnosticMap.set(targetFileName, diagnostics);
            });
          } catch (e) {
            outputChannel.appendLine(
              `Error parsing Scout output: ${
                e instanceof Error ? e.message : String(e)
              }`
            );
          }
        });

        diagnosticCollection.clear();
        for (const [file, diagnostics] of diagnosticMap) {
          const uri = vscode.Uri.file(file);
          diagnosticCollection.set(uri, diagnostics);
        }

        outputChannel.appendLine(`Scout finished`);
      } catch (error) {
        const errorMsg = `Error processing Scout output: ${
          error instanceof Error ? error.message : String(error)
        }`;
        outputChannel.appendLine(errorMsg);
        console.error(errorMsg);
      }
      resolve();
    });
  });
}

export function deactivate() {
  outputChannel.appendLine("Scout Audit extension deactivated");
  if (diagnosticCollection) {
    diagnosticCollection.clear();
    diagnosticCollection.dispose();
  }
  if (outputChannel) {
    outputChannel.dispose();
  }
}

async function checkAndInstallScout(): Promise<boolean> {
  const commandName = `cargo-scout-audit`;
  try {
    await commandExists(commandName);
    return true;
  } catch (err) {
    const userChoice = await vscode.window.showErrorMessage(
      `${commandName} not found. Please install it to proceed.`,
      "Install",
      "Cancel"
    );
    if (userChoice === "Install") {
      const terminal = vscode.window.createTerminal({
        name: `Install ${commandName}`,
      });
      terminal.show();
      terminal.sendText(
        `cargo install cargo-dylint dylint-link ${commandName}`,
        true
      );
    }
    return false;
  }
}

async function isWorkspaceValid(): Promise<boolean> {
  const workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders) return false;

  const cargoTomlPath = path.join(workspaceFolders[0].uri.fsPath, CARGO_TOML);
  try {
    await fs.access(cargoTomlPath);
    return true;
  } catch {
    return false;
  }
}
