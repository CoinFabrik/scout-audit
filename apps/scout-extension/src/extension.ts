import * as vscode from "vscode";
import commandExists from "command-exists";
import toml from "toml";
import fs from "fs";
import path from "path";

const RUST_ANALYZER_CONFIG = "rust-analyzer.check.overrideCommand";

export async function activate(_context: vscode.ExtensionContext) {
  // Check workspace is an soroban project
  if (!isProjectSoroban()) return;

  const config = vscode.workspace.getConfiguration();

  // Check rust-analyzer is installed
  if (!config.has(RUST_ANALYZER_CONFIG)) {
    console.error("rust-analyzer is not installed");
    await vscode.window.showErrorMessage(
      "rust-analyzer must be installed in order for scout to work"
    );
  }

  // Check scout is installed
  try {
    await commandExists("cargo-scout-audit");
  } catch (err) {
    console.error("cargo-scout-audit is not installed");
    await vscode.window.showErrorMessage(
      "cargo-scout-audit must be installed in order for scout to work"
    );
    return;
  }

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

function isProjectSoroban(): boolean {
  const workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders) {
    console.log("No workspace is opened.");
    return false;
  }

  // Get the path of the first workspace folder
  const cargoTomlPath = path.join(workspaceFolders[0].uri.fsPath, "Cargo.toml");
  if (!fs.existsSync(cargoTomlPath)) {
    console.log("Cargo.toml does not exist in the workspace root.");
    return false;
  }

  // Read and parse the Cargo.toml file
  const cargoToml = fs.readFileSync(cargoTomlPath, "utf-8");
  let cargoTomlParsed;
  try {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    cargoTomlParsed = toml.parse(cargoToml);
  } catch (error) {
    console.error("Error parsing Cargo.toml:", error);
    return false;
  }

  // Check if soroban-sdk is a direct dependency
  if (
    // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
    !cargoTomlParsed.dependencies ||
    // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
    !cargoTomlParsed.dependencies["soroban-sdk"]
  ) {
    console.log("soroban-sdk crate is not a direct dependency in Cargo.toml.");
    return false;
  }

  console.log("soroban-sdk crate is a direct dependency in Cargo.toml.")
  return true;
}
