/* eslint-disable @typescript-eslint/no-non-null-assertion */
import * as vscode from "vscode";
import commandExists from "command-exists";
import toml from "toml";
import fs from "fs";
import path from "path";

const RUST_ANALYZER_CONFIG = "rust-analyzer.check.overrideCommand";
const CARGO_TOML = "Cargo.toml";

enum BlockchainType {
  Ink = "ink",
  Soroban = "soroban-sdk",
}

interface CargoDependencies {
  ink?: BlockchainType.Ink;
  "soroban-sdk"?: BlockchainType.Soroban;
}

interface CargoToml {
  dependencies?: CargoDependencies;
}

export async function activate(_context: vscode.ExtensionContext) {
  // Check if the workspace is valid
  if (!isWorkspaceValid()) return;

  // Get the Cargo.toml file and parse it
  const cargoToml = await readAndParseCargoToml();
  if (!cargoToml) return;

  // Check if the Cargo.toml file has the required dependencies
  if (!(await hasKnownSdk(cargoToml))) return;

  // Get the worspace configuration and check if rust-analyzer is installed
  const config = vscode.workspace.getConfiguration();
  if (!config.has(RUST_ANALYZER_CONFIG)) {
    await vscode.window.showErrorMessage(
      "rust-analyzer is not installed. Please install rust-analyzer to continue."
    );
    return;
  }

  // Check if scout is installed
  if (!(await checkAndInstallScout())) return;

  // Update settings to change rust-analyzer config
  await config.update(RUST_ANALYZER_CONFIG, [
    "cargo",
    "scout-audit",
    "--",
    "--message-format=json",
  ]);
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

async function hasKnownSdk(cargoTomlParsed: CargoToml): Promise<boolean> {
  if (!cargoTomlParsed.dependencies) {
    await vscode.window.showErrorMessage(
      "No dependencies found in Cargo.toml. Please add 'soroban-sdk' or 'ink' as a dependency."
    );
    return false;
  }

  if (cargoTomlParsed.dependencies.ink) {
    return true;
  } else if (cargoTomlParsed.dependencies["soroban-sdk"]) {
    return true;
  } else {
    await vscode.window.showErrorMessage(
      "Neither 'soroban-sdk' nor 'ink' crates are direct dependencies in Cargo.toml."
    );
    return false;
  }
}

async function readAndParseCargoToml(): Promise<CargoToml | null> {
  const workspaceFolders = vscode.workspace.workspaceFolders!;
  const cargoTomlPath = path.join(workspaceFolders[0].uri.fsPath, CARGO_TOML);
  try {
    const cargoToml = fs.readFileSync(cargoTomlPath, "utf-8");
    return toml.parse(cargoToml) as CargoToml;
  } catch (error) {
    await vscode.window.showErrorMessage(
      "Failed to parse Cargo.toml: " + String(error)
    );
    return null;
  }
}

function isWorkspaceValid(): boolean {
  const workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders) return false;

  const cargoTomlPath = path.join(workspaceFolders[0].uri.fsPath, CARGO_TOML);
  if (!fs.existsSync(cargoTomlPath)) return false;

  return true;
}
