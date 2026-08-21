import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

/**
 * Real LSP bridge: spawns `fish lsp` (the language server bundled with the
 * Fish CLI) over stdio and lets it drive hover, completion and diagnostics for
 * `fish.toml` manifests.
 */
export class FishLSPClient {
    private client: LanguageClient | undefined;

    constructor(
        private context: vscode.ExtensionContext,
        private fishExecutable: string
    ) {}

    async start() {
        const serverOptions: ServerOptions = {
            command: this.fishExecutable,
            args: ['lsp'],
            transport: TransportKind.stdio
        };

        const documentSelector = [
            {
                scheme: 'file',
                language: 'toml',
                pattern: '**/fish.toml'
            }
        ];

        const clientOptions: LanguageClientOptions = {
            documentSelector,
            synchronize: {
                fileEvents: vscode.workspace.createFileSystemWatcher('**/fish.toml')
            },
            outputChannelName: 'Fish LSP'
        };

        this.client = new LanguageClient(
            'fish-lsp',
            'Fish Language Server',
            serverOptions,
            clientOptions
        );

        try {
            await this.client.start();
            console.log('Fish LSP client started');
        } catch (error) {
            console.error('Failed to start Fish LSP client:', error);
            vscode.window.showWarningMessage(
                'Fish LSP could not start. Make sure the `fish` executable is on your PATH (see `fish.executablePath`).'
            );
        }
    }

    stop() {
        if (this.client) {
            void this.client.stop();
            this.client = undefined;
        }
    }
}
