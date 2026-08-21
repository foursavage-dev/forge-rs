import * as vscode from 'vscode';

/**
 * Runs `fish check` (which reports build-graph and task problems) in an
 * integrated terminal. Inline `fish.toml` diagnostics (unknown keys, invalid
 * TOML) are provided by the Fish language server via `FishLSPClient`.
 */
export class FishDiagnosticsProvider {
    private fishExecutable: string;

    constructor(
        private context: vscode.ExtensionContext,
        fishExecutable: string
    ) {
        this.fishExecutable = fishExecutable;
    }

    showDiagnostics() {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        const terminal = vscode.window.createTerminal('Fish Diagnostics');
        if (workspaceFolder) {
            terminal.sendText(`cd "${workspaceFolder.uri.fsPath}"`);
        }
        terminal.sendText(`${this.fishExecutable} check`);
        terminal.show();
    }
}
