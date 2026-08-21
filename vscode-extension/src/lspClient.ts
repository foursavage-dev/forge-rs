import * as vscode from 'vscode';
import * as path from 'path';

export class FishLSPClient {
    private context: vscode.ExtensionContext;
    private fishExecutable: string;
    private diagnosticCollection: vscode.DiagnosticCollection;

    constructor(context: vscode.ExtensionContext, fishExecutable: string) {
        this.context = context;
        this.fishExecutable = fishExecutable;
        this.diagnosticCollection = vscode.languages.createDiagnosticCollection('fish-lsp');
    }

    async start() {
        console.log('Fish LSP bridge starting...');
        
        // Register for document changes to provide workspace diagnostics
        this.context.subscriptions.push(
            vscode.workspace.onDidChangeTextDocument(event => {
                this.analyzeDocument(event.document);
            })
        );

        // Analyze all open documents on startup
        for (const document of vscode.workspace.textDocuments) {
            this.analyzeDocument(document);
        }

        console.log('Fish LSP bridge started successfully');
    }

    private async analyzeDocument(document: vscode.TextDocument) {
        if (document.languageId !== 'rust') {
            return;
        }

        try {
            const diagnostics = await this.getWorkspaceDiagnostics(document);
            this.diagnosticCollection.set(document.uri, diagnostics);
        } catch (error) {
            console.error('Error analyzing document:', error);
        }
    }

    private async getWorkspaceDiagnostics(document: vscode.TextDocument): Promise<vscode.Diagnostic[]> {
        const diagnostics: vscode.Diagnostic[] = [];
        
        // For now, provide basic workspace diagnostics based on project structure
        // In a full implementation, this would call Fish's query engine
        
        const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
        if (!workspaceFolder) {
            return diagnostics;
        }

        // Check for common Fish project issues
        const hasCargoToml = await this.fileExists(path.join(workspaceFolder.uri.fsPath, 'Cargo.toml'));
        if (!hasCargoToml) {
            const range = new vscode.Range(0, 0, 0, 0);
            const diagnostic = new vscode.Diagnostic(
                range,
                'No Cargo.toml found. This might not be a Fish workspace.',
                vscode.DiagnosticSeverity.Warning
            );
            diagnostic.source = 'Fish LSP';
            diagnostics.push(diagnostic);
        }

        return diagnostics;
    }

    private async fileExists(filePath: string): Promise<boolean> {
        try {
            await vscode.workspace.fs.stat(vscode.Uri.file(filePath));
            return true;
        } catch {
            return false;
        }
    }

    stop() {
        this.diagnosticCollection.dispose();
    }
}