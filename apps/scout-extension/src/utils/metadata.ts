import * as vscode from "vscode";
import { outputChannel } from "../extension";
import { spawn } from "child_process";

interface ScoutMetadata {
  lints: {
    [key: string]: {
      id: string;
      name: string;
      severity: "Critical" | "Medium" | "Minor" | "Enhancement";
      short_message: string;
      long_message: string;
      help: string;
      vulnerability_class: string;
    };
  };
}

export let scoutMetadata: ScoutMetadata | undefined;

export async function getScoutMetadata(
  cwd: string
): Promise<ScoutMetadata | undefined> {
  if (scoutMetadata) return scoutMetadata;

  return new Promise((resolve) => {
    const scout = spawn("cargo", ["scout-audit", "--metadata"], {
      env: { ...process.env, RUST_BACKTRACE: "0" },
      cwd,
    });

    let stdout = "";
    let stderr = "";

    scout.stdout.on("data", (data: Buffer) => {
      stdout += data.toString();
    });

    scout.stderr.on("data", (data: Buffer) => {
      stderr += data.toString();
    });

    scout.on("close", (code) => {
      if (code !== 0) {
        outputChannel.appendLine(`Failed to get Scout metadata: ${stderr}`);
        resolve(undefined);
        return;
      }

      try {
        const jsonMatch = stdout.match(/\{[\s\S]*\}/);
        if (!jsonMatch) {
          outputChannel.appendLine("No metadata JSON found in Scout output");
          resolve(undefined);
          return;
        }

        const metadata = JSON.parse(jsonMatch[0]) as ScoutMetadata;
        scoutMetadata = metadata;
        resolve(metadata);
      } catch (e) {
        outputChannel.appendLine(
          `Error parsing Scout metadata: ${
            e instanceof Error ? e.message : String(e)
          }`
        );
        resolve(undefined);
      }
    });
  });
}

export function getMessageFromLint(detector: string): string {
  return scoutMetadata?.lints[detector]?.short_message || "";
}

export function getSeverityFromMetadata(
  code: string
): vscode.DiagnosticSeverity {
  if (!scoutMetadata?.lints[code]) {
    return vscode.DiagnosticSeverity.Warning;
  }

  switch (scoutMetadata.lints[code].severity) {
    case "Critical":
      return vscode.DiagnosticSeverity.Error;
    case "Medium":
      return vscode.DiagnosticSeverity.Warning;
    case "Minor":
    case "Enhancement":
      return vscode.DiagnosticSeverity.Information;
    default:
      return vscode.DiagnosticSeverity.Warning;
  }
}
