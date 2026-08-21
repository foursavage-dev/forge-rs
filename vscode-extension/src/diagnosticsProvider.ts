import * as vscode from 'vscode';
import * as path from 'path';

export class FishDiagnosticsProvider {
    private diagnosticCollection: vscode.DiagnosticCollection;
    private fishExecutable: string;

    constructor(
        private context: vscode.ExtensionContext,
        fishExecutable: string
    ) {
        this.fishExecutable = fishExecutable;
        this.diagnosticCollection = vscode.languages.createDiagnosticCollection('fish');
    }

    showDiagnostics() {
        const editor = vscode.window.activeTextEditor;
        if (editor) {
            this.analyzeDocument(editor.document);
        }
    }

    private async analyzeDocument(document: vscode.TextDocument) {
        const diagnostics: vscode.Diagnostic[] = [];
        
        // Only analyze Rust files for now
        if (document.languageId !== 'rust') {
            this.diagnosticCollection.set(document.uri, diagnostics);
            return;
        }

        try {
            // Use VS Code's terminal to run Fish check
            const terminal = vscode.window.createTerminal('Fish Diagnostics');
            terminal.sendText(`cd "${vscode.workspace.getWorkspaceFolder(document.uri)?.uri.fsPath || '.'}"`);
            terminal.sendText(`${this.fishExecutable} check`);
            
            // For now, provide basic structure diagnostics
            diagnostics.push(...this.getStructureDiagnostics(document));
            this.diagnosticCollection.set(document.uri, diagnostics);
        } catch (error) {
            console.error('Fish diagnostics error:', error);
        }
    }

    private getStructureDiagnostics(document: vscode.TextDocument): vscode.Diagnostic[] {
        const diagnostics: vscode.Diagnostic[] = [];
        const text = document.getText();
        
        // Check for common Rust issues
        if (!text.includes('mod ') && !text.includes('pub mod ')) {
            const range = new vscode.Range(0, 0, 0, 0);
            const diagnostic = new vscode.Diagnostic(
                range,
                'No module declarations found. Consider adding modules for better organization.',
                vscode.DiagnosticSeverity.Hint
            );
            diagnostic.source = 'Fish';
            diagnostics.push(diagnostic);
        }

        return diagnostics;
    }
}