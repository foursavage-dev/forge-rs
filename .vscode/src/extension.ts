import * as vscode from 'vscode';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

let buildStatusBarItem: vscode.StatusBarItem;
let cacheHitRateItem: vscode.StatusBarItem;
let packageTreeProvider: PackageTreeProvider;

export function activate(context: vscode.ExtensionContext) {
    buildStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    buildStatusBarItem.command = 'fish.build';
    buildStatusBarItem.text = '$(package) fish: Ready';
    buildStatusBarItem.show();

    cacheHitRateItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 90);
    cacheHitRateItem.command = 'fish.ui';
    cacheHitRateItem.text = '$(dashboard) fish UI';
    cacheHitRateItem.show();

    packageTreeProvider = new PackageTreeProvider();
    vscode.window.registerTreeDataProvider('fishPackages', packageTreeProvider);

    const buildCommand = vscode.commands.registerCommand('fish.build', async () => {
        await runfishCommand('build');
    });

    const testCommand = vscode.commands.registerCommand('fish.test', async () => {
        await runfishCommand('test');
    });

    const graphCommand = vscode.commands.registerCommand('fish.graph', async () => {
        await runfishCommand('graph');
    });

    const uiCommand = vscode.commands.registerCommand('fish.ui', async () => {
        await runfishCommand('ui --port 3000 --open');
    });

    const explainCommand = vscode.commands.registerCommand('fish.explain', async () => {
        await runfishCommand('build --explain');
    });

    const exportCompileCommands = vscode.commands.registerCommand('fish.exportCompileCommands', async () => {
        await runfishCommand('build');
        vscode.window.showInformationMessage('fish: compile_commands.json exported successfully for Clangd & LSP.');
    });

    const doctorCommand = vscode.commands.registerCommand('fish.doctor', async () => {
        await runfishCommand('doctor');
    });

    const affectedCommand = vscode.commands.registerCommand('fish.affected', async () => {
        await runfishCommand('affected');
    });

    const refreshCommand = vscode.commands.registerCommand('fish.refreshPackages', async () => {
        await packageTreeProvider.refresh();
    });

    const buildPackageCommand = vscode.commands.registerCommand('fish.buildPackage', async (node: PackageNode) => {
        if (node) {
            await runfishCommand(`build -p ${node.label}`);
        }
    });

    const testPackageCommand = vscode.commands.registerCommand('fish.testPackage', async (node: PackageNode) => {
        if (node) {
            await runfishCommand(`test -p ${node.label}`);
        }
    });

    context.subscriptions.push(
        buildCommand,
        testCommand,
        graphCommand,
        uiCommand,
        explainCommand,
        exportCompileCommands,
        doctorCommand,
        affectedCommand,
        refreshCommand,
        buildPackageCommand,
        testPackageCommand,
        buildStatusBarItem,
        cacheHitRateItem
    );
}

export function deactivate() {}

async function runfishCommand(command: string) {
    const config = vscode.workspace.getConfiguration('fish');
    const fishPath = config.get<string>('path', 'fish');
    const rootPath = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || '.';

    buildStatusBarItem.text = '$(sync~spin) fish: Running...';

    const outputChannel = vscode.window.createOutputChannel('fish');
    outputChannel.show();
    outputChannel.appendLine(`> ${fishPath} ${command}`);

    try {
        const { stdout, stderr } = await execAsync(`${fishPath} ${command}`, { cwd: rootPath });
        if (stdout) outputChannel.append(stdout);
        if (stderr) outputChannel.append(stderr);
        buildStatusBarItem.text = '$(check) fish: Succeeded';
        vscode.window.showInformationMessage(`fish: Command completed successfully.`);
    } catch (err: any) {
        if (err.stdout) outputChannel.append(err.stdout);
        if (err.stderr) outputChannel.append(err.stderr);
        buildStatusBarItem.text = '$(error) fish: Failed';
        vscode.window.showErrorMessage(`fish: Command failed. Check fish output panel.`);
    }
}

class PackageNode extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly dependencies: string[] = []
    ) {
        super(label, collapsibleState);
        this.contextValue = 'fishPackage';
        this.iconPath = new vscode.ThemeIcon('package');
        this.tooltip = `${label} (${dependencies.length} direct dependencies)`;
    }
}

class PackageTreeProvider implements vscode.TreeDataProvider<PackageNode> {
    private _onDidChangeTreeData: vscode.EventEmitter<PackageNode | undefined | void> = new vscode.EventEmitter<PackageNode | undefined | void>();
    readonly onDidChangeTreeData: vscode.Event<PackageNode | undefined | void> = this._onDidChangeTreeData.event;

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: PackageNode): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: PackageNode): Promise<PackageNode[]> {
        if (element) {
            return element.dependencies.map(
                dep => new PackageNode(dep, vscode.TreeItemCollapsibleState.None)
            );
        }

        const config = vscode.workspace.getConfiguration('fish');
        const fishPath = config.get<string>('path', 'fish');
        const rootPath = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || '.';

        try {
            const { stdout } = await execAsync(`${fishPath} graph --format json`, { cwd: rootPath });
            const data = JSON.parse(stdout);
            const packages = data.packages || data.nodes || [];
            return packages.map((pkg: any) => {
                const name = pkg.name || pkg.id || 'unknown';
                const deps = pkg.dependencies || [];
                const state = deps.length > 0
                    ? vscode.TreeItemCollapsibleState.Collapsed
                    : vscode.TreeItemCollapsibleState.None;
                return new PackageNode(name, state, deps);
            });
        } catch {
            return [new PackageNode('Workspace Packages (Run fish: Refresh)', vscode.TreeItemCollapsibleState.None)];
        }
    }
}