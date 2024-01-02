import * as vscode from "vscode";
import commandExists from "command-exists";
import toml from "toml";
import fs from "fs";
import path from "path";

const RUST_ANALYZER_CONFIG = "rust-analyzer.check.overrideCommand";

export async function activate(_context: vscode.ExtensionContext) {
  // Check workspace is an soroban project
  const sdk = isProjectSupported();
  if (sdk == false) return;

  const config = vscode.workspace.getConfiguration();

  // Check rust-analyzer is installed
  if (!config.has(RUST_ANALYZER_CONFIG)) {
    console.error("rust-analyzer is not installed");
    await vscode.window.showErrorMessage(
      "rust-analyzer must be installed in order for scout to work"
    );
  }

  // Check scout is installed

  if(sdk == "ink") {
    console.log("scout-audit")
    try {
      await commandExists("cargo-scout-audit");
    } catch (err) {
      console.error("cargo-scout-audit is not installed");
      await vscode.window.showErrorMessage(
        "cargo-scout-audit must be installed in order for scout to work"
      );
      return false;
    }
    
    // Update settings to change rust-analyzer config
    await config.update(RUST_ANALYZER_CONFIG, [
      "cargo",
      "scout-audit",
      "--",
      "--message-format=json",
    ]);
  } else if(sdk == "soroban-sdk") {
    try {
      await commandExists("cargo-scout-audit-soroban");
    } catch (err) {
      console.error("cargo-scout-audit-soroban is not installed");
      await vscode.window.showErrorMessage(
        "cargo-scout-audit-soroban must be installed in order for scout to work"
      );
      return false;
    }
    
    // Update settings to change rust-analyzer config
    await config.update(RUST_ANALYZER_CONFIG, [
      "cargo",
      "scout-audit-soroban",
      "--",
      "--message-format=json",
    ]);
  }
}

export function deactivate() {
  // unused
}

function isProjectSupported(): false|string {
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

  // Check if soroban-sdk or ink are a direct dependency

  // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
  if(cargoTomlParsed?.dependencies?.ink) {
    console.log("ink crate is a direct dependency in Cargo.toml.")
    return "ink";
  }
  // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
  if(cargoTomlParsed?.dependencies?.["soroban-sdk"]) {
    console.log("soroban-sdk crate is a direct dependency in Cargo.toml.")
    return "soroban-sdk";
  }

  console.log("soroban-sdk or ink crates are not a direct dependency in Cargo.toml.");
  return false;
}
