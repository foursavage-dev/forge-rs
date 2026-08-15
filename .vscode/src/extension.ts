import * as vscode from 'vscode';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

interface BuildStatus {
    status: 'idle' | 'building' | 'success' | 'failed';
    cacheHitRate?: number;
    packageCount?: number;
}

let buildStatusBarItem: vscode.StatusBarItem;
let cacheHitRateItem: vscode.StatusBarItem;
let packageTreeProvider: PackageTreeProvider;

export function activate(context: vscode.ExtensionContext) {
    console.log('Forge extension is now active!');

    // Create status bar items
    buildStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    buildStatusBarItem.command = 'forge.build';
    buildStatusBarItem.text = '$(package) Forge: Idle';
    buildStatusBarItem.show();

    cacheHitRateItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 90);
    cacheHitRateItem.text = '$(database) Cache: 0%';
    cacheHitRateItem.show();

    // Initialize package tree provider
    packageTreeProvider = new PackageTreeProvider();
    vscode.window.registerTreeDataProvider('forgePackages', packageTreeProvider);

    // Build command
    const buildCommand = vscode.commands.registerCommand('forge.build', async () => {
        await runForgeCommand('build');
    });

    // Test command
    const testCommand = vscode.commands.registerCommand('forge.test', async () => {
        await runForgeCommand('test');
    });

    // Graph command
    const graphCommand = vscode.commands.registerCommand('forge.graph', async () => {
        await runForgeCommand('graph');
    });

    // Clean command
    const cleanCommand = vscode.commands.registerCommand('forge.clean', async () => {
        await runForgeCommand('clean');
    });

    // Doctor command
    const doctorCommand = vscode.commands.registerCommand('forge.doctor', async () => {
        await runForgeCommand('doctor');
    });

    // Affected command
    const affectedCommand = vscode.commands.registerCommand('forge.affected', async () => {
        await runForgeCommand('affected');
    });

    // Refresh packages command
    const refreshCommand = vscode.commands.registerCommand('forge.refreshPackages', async () => {
        await packageTreeProvider.refresh();
    });

    // Build specific package command
    const buildPackageCommand = vscode.commands.registerCommand('forge.buildPackage', async (node: PackageNode) => {
        if (node) {
            await runForgeCommand(`build --package ${node.label}`);
        }
    });

    context.subscriptions.push(
        buildCommand,
        testCommand,
        graphCommand,
        cleanCommand,
        doctorCommand,
        affectedCommand,
        refreshCommand,
        buildPackageCommand,
        buildStatusBarItem,
        cacheHitRateItem
    );
}

async function runForgeCommand(command: string) {
    const config = vscode.workspace.getConfiguration('forge');
    const forgePath = config.get<string>('path', 'forge');
    const experimental = config.get<boolean>('experimental', false);
    const maxJobs = config.get<number>('maxJobs', 4);

    let cmd = `${forgePath} ${command}`;
    
    if (experimental) {
        cmd += ' --experimental';
    }
    
    if (maxJobs > 0) {
        cmd += ` -j ${maxJobs}`;
    }

    try {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            vscode.window.showErrorMessage('No workspace folder found');
            return;
        }

        // Update status bar to building
        buildStatusBarItem.text = '$(sync~spin) Forge: Building...';
        buildStatusBarItem.backgroundColor = new vscode.ThemeColor('statusBar.warningBackground');

        const terminal = vscode.window.createTerminal(`Forge ${command}`);
        terminal.sendText(`cd "${workspaceFolder.uri.fsPath}"`);
        terminal.sendText(cmd);
        terminal.show();

        // Monitor terminal output for build status
        monitorBuildOutput(terminal);

    } catch (error) {
        vscode.window.showErrorMessage(`Forge command failed: ${error}`);
        buildStatusBarItem.text = '$(x) Forge: Failed';
        buildStatusBarItem.backgroundColor = new vscode.ThemeColor('statusBar.errorBackground');
    }
}

function monitorBuildOutput(terminal: vscode.Terminal) {
    // In a real implementation, you would parse terminal output
    // For now, we'll simulate completion after a timeout
    setTimeout(() => {
        buildStatusBarItem.text = '$(check) Forge: Success';
        buildStatusBarItem.backgroundColor = new vscode.ThemeColor('statusBar.successBackground');
        cacheHitRateItem.text = '$(database) Cache: 85%';
    }, 3000);
}

class PackageTreeProvider implements vscode.TreeDataProvider<PackageNode> {
    private _onDidChangeTreeData = new vscode.EventEmitter<PackageNode | undefined | null | void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    private packages: PackageNode[] = [];

    constructor() {
        this.loadPackages();
    }

    refresh(): void {
        this.loadPackages();
        this._onDidChangeTreeData.fire(undefined);
    }

    private async loadPackages() {
        // In a real implementation, this would parse Cargo.toml or forge configuration
        // For now, we'll create mock package data
        this.packages = [
            new PackageNode('forge-core', vscode.TreeItemCollapsibleState.Collapsed),
            new PackageNode('forge-cli', vscode.TreeItemCollapsibleState.Collapsed),
            new PackageNode('forge-worker', vscode.TreeItemCollapsibleState.Collapsed),
            new PackageNode('forge-cache', vscode.TreeItemCollapsibleState.Collapsed),
        ];
    }

    getTreeItem(element: PackageNode): vscode.TreeItem {
        return element;
    }

    getChildren(element?: PackageNode): Thenable<PackageNode[]> {
        if (!element) {
            return Promise.resolve(this.packages);
        }
        
        // Return tasks for a package
        const tasks = [
            new PackageNode('build', vscode.TreeItemCollapsibleState.None, element.label),
            new PackageNode('test', vscode.TreeItemCollapsibleState.None, element.label),
            new PackageNode('clean', vscode.TreeItemCollapsibleState.None, element.label),
        ];
        return Promise.resolve(tasks);
    }
}

class PackageNode extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly parentPackage?: string
    ) {
        super(label, collapsibleState);
        
        if (parentPackage) {
            this.contextValue = 'task';
            this.command = {
                command: 'forge.buildPackage',
                title: 'Build',
                arguments: [this]
            };
        } else {
            this.contextValue = 'package';
            this.iconPath = new vscode.ThemeIcon('package');
        }
    }
}

export function deactivate() {
    console.log('Forge extension is now deactivated');
    if (buildStatusBarItem) {
        buildStatusBarItem.dispose();
    }
    if (cacheHitRateItem) {
        cacheHitRateItem.dispose();
    }
}