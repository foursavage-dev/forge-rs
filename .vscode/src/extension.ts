import * as vscode from 'vscode';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

let buildStatusBarItem: vscode.StatusBarItem;
let cacheHitRateItem: vscode.StatusBarItem;
let packageTreeProvider: PackageTreeProvider;

export function activate(context: vscode.ExtensionContext) {
    buildStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    buildStatusBarItem.command = 'forge.build';
    buildStatusBarItem.text = '$(package) Forge: Ready';
    buildStatusBarItem.show();

    cacheHitRateItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 90);
    cacheHitRateItem.command = 'forge.ui';
    cacheHitRateItem.text = '$(dashboard) Forge UI';
    cacheHitRateItem.show();

    packageTreeProvider = new PackageTreeProvider();
    vscode.window.registerTreeDataProvider('forgePackages', packageTreeProvider);

    const buildCommand = vscode.commands.registerCommand('forge.build', async () => {
        await runForgeCommand('build');
    });

    const testCommand = vscode.commands.registerCommand('forge.test', async () => {
        await runForgeCommand('test');
    });

    const graphCommand = vscode.commands.registerCommand('forge.graph', async () => {
        await runForgeCommand('graph');
    });

    const uiCommand = vscode.commands.registerCommand('forge.ui', async () => {
        await runForgeCommand('ui --port 3000 --open');
    });

    const explainCommand = vscode.commands.registerCommand('forge.explain', async () => {
        await runForgeCommand('build --explain');
    });

    const exportCompileCommands = vscode.commands.registerCommand('forge.exportCompileCommands', async () => {
        await runForgeCommand('build');
        vscode.window.showInformationMessage('Forge: compile_commands.json exported successfully for Clangd & LSP.');
    });

    const doctorCommand = vscode.commands.registerCommand('forge.doctor', async () => {
        await runForgeCommand('doctor');
    });

    const affectedCommand = vscode.commands.registerCommand('forge.affected', async () => {
        await runForgeCommand('affected');
    });

    const refreshCommand = vscode.commands.registerCommand('forge.refreshPackages', async () => {
        await packageTreeProvider.refresh();
    });

    const buildPackageCommand = vscode.commands.registerCommand('forge.buildPackage', async (node: PackageNode) => {
        if (node) {
            await runForgeCommand(`build -p ${node.label}`);
        }
    });

    const testPackageCommand = vscode.commands.registerCommand('forge.testPackage', async (node: PackageNode) => {
        if (node) {
            await runForgeCommand(`test -p ${node.label}`);
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

async function runForgeCommand(command: string) {
    const config = vscode.workspace.getConfiguration('forge');
    const forgePath = config.get<string>('path', 'forge');
    const rootPath = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || '.';

    buildStatusBarItem.text = '$(sync~spin) Forge: Running...';

    const outputChannel = vscode.window.createOutputChannel('Forge');
    outputChannel.show();
    outputChannel.appendLine(`> ${forgePath} ${command}`);

    try {
        const { stdout, stderr } = await execAsync(`${forgePath} ${command}`, { cwd: rootPath });
        if (stdout) outputChannel.append(stdout);
        if (stderr) outputChannel.append(stderr);
        buildStatusBarItem.text = '$(check) Forge: Succeeded';
        vscode.window.showInformationMessage(`Forge: Command completed successfully.`);
    } catch (err: any) {
        if (err.stdout) outputChannel.append(err.stdout);
        if (err.stderr) outputChannel.append(err.stderr);
        buildStatusBarItem.text = '$(error) Forge: Failed';
        vscode.window.showErrorMessage(`Forge: Command failed. Check Forge output panel.`);
    }
}

class PackageNode extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly dependencies: string[] = []
    ) {
        super(label, collapsibleState);
        this.contextValue = 'forgePackage';
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

        const config = vscode.workspace.getConfiguration('forge');
        const forgePath = config.get<string>('path', 'forge');
        const rootPath = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || '.';

        try {
            const { stdout } = await execAsync(`${forgePath} graph --format json`, { cwd: rootPath });
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
            return [new PackageNode('Workspace Packages (Run Forge: Refresh)', vscode.TreeItemCollapsibleState.None)];
        }
    }
}