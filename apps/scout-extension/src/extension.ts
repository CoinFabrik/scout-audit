import * as vscode from "vscode";
import commandExists from "command-exists";
import { parse, type TomlTable } from "@iarna/toml";
import fs from "fs/promises";
import path from "path";

const RUST_ANALYZER_CONFIG = "rust-analyzer.check.overrideCommand";
const CARGO_TOML = "Cargo.toml";

interface CargoToml extends TomlTable {
  dependencies?: Record<string, unknown>;
}

export async function activate(_context: vscode.ExtensionContext) {
  try {
    if (!(await isWorkspaceValid())) {
      await vscode.window.showErrorMessage(
        "Invalid workspace: Cargo.toml not found."
      );
      return;
    }

    const cargoToml = await readAndParseCargoToml();
    if (!cargoToml) return;

    const config = vscode.workspace.getConfiguration();
    if (!config.has(RUST_ANALYZER_CONFIG)) {
      await vscode.window.showErrorMessage(
        "rust-analyzer is not installed. Please install rust-analyzer to continue."
      );
      return;
    }

    if (!(await checkAndInstallScout())) return;

    const newConfig = ["cargo", "scout-audit", "--", "--message-format=json"];
    const currentConfig = config.get(RUST_ANALYZER_CONFIG);
    if (JSON.stringify(currentConfig) !== JSON.stringify(newConfig)) {
      await config.update(
        RUST_ANALYZER_CONFIG,
        newConfig,
        vscode.ConfigurationTarget.Workspace
      );
      await vscode.window.showInformationMessage(
        "Updated rust-analyzer configuration."
      );
    }
  } catch (error) {
    await vscode.window.showErrorMessage(
      `Activation error: ${
        error instanceof Error ? error.message : String(error)
      }`
    );
  }
}

export function deactivate() {
  // unused
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

async function readAndParseCargoToml(): Promise<CargoToml | null> {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const workspaceFolders = vscode.workspace.workspaceFolders!;

  const cargoTomlPath = path.join(workspaceFolders[0].uri.fsPath, CARGO_TOML);
  try {
    const cargoToml = await fs.readFile(cargoTomlPath, "utf-8");
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call
    const parsed = parse(cargoToml);
    return parsed as CargoToml;
  } catch (error) {
    await vscode.window.showErrorMessage(
      `Failed to parse Cargo.toml: ${
        error instanceof Error ? error.message : String(error)
      }`
    );
    return null;
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
