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
let statusBarItem: vscode.StatusBarItem;

export function activate(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('fish');
    const fishExecutable = config.get<string>('executablePath', 'fish');

    dagProvider = new FishDAGProvider(context, fishExecutable);
    taskProvider = new FishTaskProvider(context, fishExecutable);
    diagnosticsProvider = new FishDiagnosticsProvider(context, fishExecutable);
    lspClient = new FishLSPClient(context, fishExecutable);
    dagVisualizer = new FishDAGVisualizer(context, fishExecutable);

    const dagTreeView = vscode.window.createTreeView('fishDAGView', {
        treeDataProvider: dagProvider,
        showCollapseAll: true
    });

    const tasksTreeView = vscode.window.createTreeView('fishTasksView', {
        treeDataProvider: taskProvider,
        showCollapseAll: true
    });

    context.subscriptions.push(dagTreeView, tasksTreeView);

    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 50);
    statusBarItem.text = '$(flame) Fish';
    statusBarItem.tooltip = 'Fish Build Orchestration (Click for Actions)';
    statusBarItem.command = 'fish.quickMenu';
    statusBarItem.show();
    context.subscriptions.push(statusBarItem);

    context.subscriptions.push(
        vscode.commands.registerCommand('fish.runBuild', () => runFishCommand('build')),
        vscode.commands.registerCommand('fish.runTest', () => runFishCommand('test')),
        vscode.commands.registerCommand('fish.clean', () => runFishCommand('clean')),
        vscode.commands.registerCommand('fish.openDashboard', () => openWebDashboard()),
        vscode.commands.registerCommand('fish.runDoctor', () => runDoctor()),
        vscode.commands.registerCommand('fish.quickMenu', () => showQuickMenu()),
        vscode.commands.registerCommand('fish.refreshGraph', () => {
            dagProvider.refresh();
            dagVisualizer.show();
        }),
        vscode.commands.registerCommand('fish.showDiagnostics', () => diagnosticsProvider?.showDiagnostics()),
        vscode.commands.registerCommand('fish.watch', () => toggleWatchMode())
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('fish.runPackageBuild', (item) => runPackageCommand(item, 'build')),
        vscode.commands.registerCommand('fish.runPackageTest', (item) => runPackageCommand(item, 'test')),
        vscode.commands.registerCommand('fish.runPackageClean', (item) => runPackageCommand(item, 'clean'))
    );

    updateWorkspaceContext();
    context.subscriptions.push(
        vscode.workspace.onDidChangeWorkspaceFolders(() => updateWorkspaceContext())
    );

    if (config.get<boolean>('autoRefresh', true)) {
        const watcher = vscode.workspace.createFileSystemWatcher('**/*.{toml,json,rs,go,ts,py,java,cs,swift,dart,zig,dockerfile}');
        context.subscriptions.push(watcher);

        watcher.onDidChange(() => {
            dagProvider.refresh();
            taskProvider.refresh();
        });
    }

    void lspClient.start();
}

function updateWorkspaceContext() {
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

async function showQuickMenu() {
    const items: vscode.QuickPickItem[] = [
        { label: '$(play) Run Build', description: 'fish build', detail: 'Build all workspace packages and tasks' },
        { label: '$(beaker) Run Tests', description: 'fish test', detail: 'Run all workspace test suites' },
        { label: '$(dashboard) Open Web Dashboard', description: 'fish dashboard', detail: 'Open interactive Web UI & DAG visualizer in browser' },
        { label: '$(graph) View DAG Graph', description: 'fish graph', detail: 'Visualize dependency DAG within VS Code' },
        { label: '$(pulse) Run Doctor Diagnostics', description: 'fish doctor', detail: 'Check system health, cache integrity and toolchains' },
        { label: '$(eye) Toggle Watch Mode', description: 'fish watch', detail: 'Continuous build on file changes' },
        { label: '$(trash) Clean Cache', description: 'fish clean', detail: 'Clear build artifacts and fingerprints' }
    ];

    const selected = await vscode.window.showQuickPick(items, {
        placeHolder: 'Select a Fish Build Orchestration action'
    });

    if (!selected) {
        return;
    }

    if (selected.label.includes('Run Build')) {
        await runFishCommand('build');
    } else if (selected.label.includes('Run Tests')) {
        await runFishCommand('test');
    } else if (selected.label.includes('Open Web Dashboard')) {
        await openWebDashboard();
    } else if (selected.label.includes('View DAG Graph')) {
        dagVisualizer.show();
    } else if (selected.label.includes('Run Doctor')) {
        await runDoctor();
    } else if (selected.label.includes('Toggle Watch')) {
        await toggleWatchMode();
    } else if (selected.label.includes('Clean Cache')) {
        await runFishCommand('clean');
    }
}

async function openWebDashboard() {
    const config = vscode.workspace.getConfiguration('fish');
    const fishExecutable = config.get<string>('executablePath', 'fish');
    const terminal = vscode.window.createTerminal('Fish Dashboard');
    terminal.sendText(`${fishExecutable} dashboard --open`);
    terminal.show();
}

async function runDoctor() {
    const config = vscode.workspace.getConfiguration('fish');
    const fishExecutable = config.get<string>('executablePath', 'fish');
    const terminal = vscode.window.createTerminal('Fish Doctor');
    terminal.sendText(`${fishExecutable} doctor`);
    terminal.show();
}

async function runFishCommand(command: string) {
    const config = vscode.workspace.getConfiguration('fish');
    const fishExecutable = config.get<string>('executablePath', 'fish');
    const showNotifications = config.get<boolean>('showBuildNotifications', true);

    if (statusBarItem) {
        statusBarItem.text = `$(sync~spin) Fish: ${command}...`;
    }

    if (showNotifications) {
        await vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: `Fish: Running ${command}...`,
            cancellable: false
        }, async () => {
            try {
                await executeFishCommand(fishExecutable, command);
                if (statusBarItem) {
                    statusBarItem.text = '$(check) Fish: Ready';
                }
                vscode.window.showInformationMessage(`Fish ${command} completed successfully`);
            } catch (error) {
                if (statusBarItem) {
                    statusBarItem.text = '$(error) Fish: Error';
                }
                vscode.window.showErrorMessage(`Fish ${command} failed: ${error}`);
            }
        });
    } else {
        try {
            await executeFishCommand(fishExecutable, command);
            if (statusBarItem) {
                statusBarItem.text = '$(check) Fish: Ready';
            }
        } catch (error) {
            if (statusBarItem) {
                statusBarItem.text = '$(error) Fish: Error';
            }
            vscode.window.showErrorMessage(`Fish ${command} failed: ${error}`);
        }
    }
}

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
    if (statusBarItem) {
        statusBarItem.dispose();
    }
}
