import * as vscode from 'vscode';
import * as path from 'path';
import { FishDAGProvider } from './dagProvider';
import { FishTaskProvider } from './taskProvider';
import { FishDiagnosticsProvider } from './diagnosticsProvider';
import { FishLSPClient } from './lspClient';
import { FishDAGVisualizer } from './dagVisualizer';

let dagProvider: FishDAGProvider;
let taskProvider: FishTaskProvider;
let diagnosticsProvider: FishDiagnosticsProvider;
let lspClient: FishLSPClient;
let dagVisualizer: FishDAGVisualizer;

export function activate(context: vscode.ExtensionContext) {
    console.log('Fish Build Orchestration extension is now active!');

    // Get configuration
    const config = vscode.workspace.getConfiguration('fish');
    const fishExecutable = config.get<string>('executablePath', 'fish');

    // Initialize providers
    dagProvider = new FishDAGProvider(context, fishExecutable);
    taskProvider = new FishTaskProvider(context, fishExecutable);
    diagnosticsProvider = new FishDiagnosticsProvider(context, fishExecutable);
    lspClient = new FishLSPClient(context, fishExecutable);
    dagVisualizer = new FishDAGVisualizer(context, fishExecutable);

    // Register tree data providers
    const dagTreeView = vscode.window.createTreeView('fishDAGView', {
        treeDataProvider: dagProvider,
        showCollapseAll: true
    });

    const tasksTreeView = vscode.window.createTreeView('fishTasksView', {
        treeDataProvider: taskProvider,
        showCollapseAll: true
    });



    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('fish.runBuild', () => runFishCommand('build')),
        vscode.commands.registerCommand('fish.runTest', () => runFishCommand('test')),
        vscode.commands.registerCommand('fish.clean', () => runFishCommand('clean')),
        vscode.commands.registerCommand('fish.refreshGraph', () => dagProvider.refresh()),
        vscode.commands.registerCommand('fish.showDiagnostics', () => diagnosticsProvider?.showDiagnostics()),
        vscode.commands.registerCommand('fish.watch', () => toggleWatchMode())
    );

    // Register tree view commands
    context.subscriptions.push(
        vscode.commands.registerCommand('fish.runPackageBuild', (item) => runPackageCommand(item, 'build')),
        vscode.commands.registerCommand('fish.runPackageTest', (item) => runPackageCommand(item, 'test')),
        vscode.commands.registerCommand('fish.runPackageClean', (item) => runPackageCommand(item, 'clean'))
    );

    // Register context key for workspace detection
    updateWorkspaceContext();
    context.subscriptions.push(
        vscode.workspace.onDidChangeWorkspaceFolders(() => updateWorkspaceContext())
    );

    // Watch for file changes if auto-refresh is enabled
    if (config.get<boolean>('autoRefresh', true)) {
        const watcher = vscode.workspace.createFileSystemWatcher('**/*.{toml,json,rs,go,ts,py,java,cs,swift,dart,zig,dockerfile}');
        context.subscriptions.push(watcher);
        
        watcher.onDidChange(() => {
            dagProvider.refresh();
            taskProvider.refresh();
        });
    }

    // Start LSP client
    lspClient.start();
}

function updateWorkspaceContext() {
    const hasFishProject = vscode.workspace.workspaceFolders?.some(folder => {
        const manifestPath = path.join(folder.uri.fsPath, 'Cargo.toml');
        return vscode.workspace.fs.stat(vscode.Uri.file(manifestPath)).then(() => true, () => false);
    }) || false;
    
    vscode.commands.executeCommand('setContext', 'workspaceHasFishProject', hasFishProject);
}

async function runFishCommand(command: string) {
    const config = vscode.workspace.getConfiguration('fish');
    const showNotifications = config.get<boolean>('showBuildNotifications', true);

    if (showNotifications) {
        vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: `Fish: Running ${command}...`,
            cancellable: false
        }, async (progress) => {
            try {
                await executeFishCommand(command, progress);
                vscode.window.showInformationMessage(`Fish ${command} completed successfully`);
            } catch (error) {
                vscode.window.showErrorMessage(`Fish ${command} failed: ${error}`);
            }
        });
    } else {
        try {
            await executeFishCommand(command, undefined);
        } catch (error) {
            vscode.window.showErrorMessage(`Fish ${command} failed: ${error}`);
        }
    }
}

async function executeFishCommand(command: string, progress?: vscode.Progress<{ message?: string }>) {
    const terminal = vscode.window.createTerminal('Fish Build');
    terminal.sendText(`fish ${command}`);
    
    return new Promise<void>((resolve, reject) => {
        const disposable = vscode.window.onDidCloseTerminal(event => {
            if (event === terminal) {
                disposable.dispose();
                resolve();
            }
        });
    });
}

async function runPackageCommand(item: any, command: string) {
    const packageName = item.label;
    vscode.window.showInformationMessage(`Running ${command} for ${packageName}...`);
    // Implementation for package-specific commands
}

async function toggleWatchMode() {
    const terminal = vscode.window.createTerminal('Fish Watch');
    terminal.sendText('fish watch');
}

export function deactivate() {
    if (lspClient) {
        lspClient.stop();
    }
}