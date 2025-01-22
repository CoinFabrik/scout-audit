import * as vscode from "vscode";
import { EXTENSION_NAME } from "../extension";

let statusBarItem: vscode.StatusBarItem;

type StatusBarState = "active" | "inactive" | "error" | "loading";

export function initStatusBar(context: vscode.ExtensionContext) {
  statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    1000
  );
  context.subscriptions.push(statusBarItem);
}

export function disposeStatusBar() {
  if (statusBarItem) {
    statusBarItem.dispose();
  }
}

export function updateStatusBar(state: StatusBarState) {
  if (state === "active") {
    statusBarItem.text = "$(shield) Scout Audit";
    statusBarItem.tooltip = "Scout Audit is active - Click to run manual audit";
    statusBarItem.command = `${EXTENSION_NAME}.run`;
  } else if (state === "inactive") {
    statusBarItem.text = "$(shield-x) Scout Audit";
    statusBarItem.tooltip =
      "Scout Audit is inactive - No relevant dependencies found";
    statusBarItem.command = undefined;
  } else if (state === "error") {
    statusBarItem.text = "$(error) Scout Audit";
    statusBarItem.tooltip = "Scout Audit encountered an error";
    statusBarItem.command = undefined;
  } else if (state === "loading") {
    statusBarItem.text = "$(sync~spin) Scout Audit";
    statusBarItem.tooltip = "Scout Audit is running...";
    statusBarItem.command = `${EXTENSION_NAME}.run`;
  }
  statusBarItem.show();
}
