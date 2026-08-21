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

    context.subscriptions.push(dagTreeView, tasksTreeView);

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('fish.runBuild', () => runFishCommand('build')),
        vscode.commands.registerCommand('fish.runTest', () => runFishCommand('test')),
        vscode.commands.registerCommand('fish.clean', () => runFishCommand('clean')),
        vscode.commands.registerCommand('fish.refreshGraph', () => {
            dagProvider.refresh();
            dagVisualizer.show();
        }),
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
    void lspClient.start();
}

function updateWorkspaceContext() {
    // A workspace "has a Fish project" when it contains either a `fish.toml`
    // manifest or a Cargo workspace (the Rust backend works without a manifest).
    const hasFishProject = vscode.workspace.workspaceFolders?.some(folder => {
        const fishManifest = path.join(folder.uri.fsPath, 'fish.toml');
        const cargoManifest = path.join(folder.uri.fsPath, 'Cargo.toml');
        return vscode.workspace.fs.stat(vscode.Uri.file(fishManifest))
            .then(() => true, () =>
                vscode.workspace.fs.stat(vscode.Uri.file(cargoManifest)).then(() => true, () => false)
            );
    }) || false;

    void vscode.commands.executeCommand('setContext', 'workspaceHasFishProject', hasFishProject);
}

async function runFishCommand(command: string) {
    const config = vscode.workspace.getConfiguration('fish');
    const fishExecutable = config.get<string>('executablePath', 'fish');
    const showNotifications = config.get<boolean>('showBuildNotifications', true);

    if (showNotifications) {
        await vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: `Fish: Running ${command}...`,
            cancellable: false
        }, async () => {
            try {
                await executeFishCommand(fishExecutable, command);
                vscode.window.showInformationMessage(`Fish ${command} completed successfully`);
            } catch (error) {
                vscode.window.showErrorMessage(`Fish ${command} failed: ${error}`);
            }
        });
    } else {
        try {
            await executeFishCommand(fishExecutable, command);
        } catch (error) {
            vscode.window.showErrorMessage(`Fish ${command} failed: ${error}`);
        }
    }
}

/**
 * Run `<fishExecutable> <args...>` as a task and resolve/reject when the
 * spawned process exits. Uses a `ShellExecution` task so completion is
 * detected from the process exit code rather than waiting for the user to
 * close a terminal.
 */
function executeFishCommand(fishExecutable: string, ...args: string[]): Promise<void> {
    return new Promise<void>((resolve, reject) => {
        const commandLine = [fishExecutable, ...args].join(' ');
        const task = new vscode.Task(
            { type: 'fish' },
            vscode.TaskScope.Workspace,
            `fish ${args[0] ?? ''}`.trim(),
            'fish',
            new vscode.ShellExecution(commandLine)
        );
        // Compare against the task identity (`event.execution.task === task`)
        // rather than the `TaskExecution` returned by `executeTask`, which is
        // a `Thenable` and would race with the first process-exit event.
        const disposable = vscode.tasks.onDidEndTaskProcess((event) => {
            if (event.execution.task === task) {
                disposable.dispose();
                if (event.exitCode === 0) {
                    resolve();
                } else {
                    reject(new Error(`exit code ${event.exitCode}`));
                }
            }
        });
        void vscode.tasks.executeTask(task);
    });
}

async function runPackageCommand(item: any, command: string) {
    const config = vscode.workspace.getConfiguration('fish');
    const fishExecutable = config.get<string>('executablePath', 'fish');
    const packageDir = item?.path ?? item?.label;

    if (!packageDir) {
        vscode.window.showErrorMessage('Could not determine the package directory.');
        return;
    }

    // `fish build/check/test` accept a positional start directory, so passing
    // the package directory builds the graph rooted there.
    try {
        await executeFishCommand(fishExecutable, command, packageDir);
        vscode.window.showInformationMessage(`Fish ${command} completed for ${item.label}`);
    } catch (error) {
        vscode.window.showErrorMessage(`Fish ${command} failed for ${item.label}: ${error}`);
    }
}

async function toggleWatchMode() {
    const config = vscode.workspace.getConfiguration('fish');
    const fishExecutable = config.get<string>('executablePath', 'fish');
    const terminal = vscode.window.createTerminal('Fish Watch');
    terminal.sendText(`${fishExecutable} watch`);
    terminal.show();
}

export function deactivate() {
    if (lspClient) {
        lspClient.stop();
    }
}
