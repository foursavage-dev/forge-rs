import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs/promises';

export class FishDAGProvider implements vscode.TreeDataProvider<DAGNode> {
    private _onDidChangeTreeData = new vscode.EventEmitter<DAGNode | undefined | null | void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    constructor(
        private context: vscode.ExtensionContext,
        private fishExecutable: string
    ) {}

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: DAGNode): vscode.TreeItem {
        const treeItem = new vscode.TreeItem(element.label, element.collapsibleState);
        treeItem.iconPath = this.getIconForNode(element);
        treeItem.contextValue = element.type;
        treeItem.command = {
            command: 'fish.runPackageBuild',
            title: 'Run Build',
            arguments: [element]
        };
        return treeItem;
    }

    async getChildren(element?: DAGNode): Promise<DAGNode[]> {
        if (!element) {
            return this.getRootNodes();
        }
        return this.getChildNodes(element);
    }

    private async getRootNodes(): Promise<DAGNode[]> {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders) {
            return [new DAGNode('No workspace found', vscode.TreeItemCollapsibleState.None, 'error')];
        }

        const nodes: DAGNode[] = [];
        
        for (const folder of workspaceFolders) {
            const cargoPath = path.join(folder.uri.fsPath, 'Cargo.toml');
            try {
                await fs.access(cargoPath);
                const projectNode = new DAGNode(
                    folder.name,
                    vscode.TreeItemCollapsibleState.Collapsed,
                    'workspace',
                    folder.uri.fsPath
                );
                nodes.push(projectNode);
            } catch {
                // Not a Rust project, skip
            }
        }

        return nodes.length > 0 ? nodes : [new DAGNode('No Fish projects found', vscode.TreeItemCollapsibleState.None, 'error')];
    }

    private async getChildNodes(element: DAGNode): Promise<DAGNode[]> {
        if (element.type === 'workspace') {
            return this.getPackageNodes(element.path!);
        }
        return [];
    }

    private async getPackageNodes(workspacePath: string): Promise<DAGNode[]> {
        const packages: DAGNode[] = [];
        const cratesPath = path.join(workspacePath, 'crates');
        
        try {
            const entries = await fs.readdir(cratesPath);
            
            for (const entry of entries) {
                const packagePath = path.join(cratesPath, entry);
                const stat = await fs.stat(packagePath);
                
                if (stat.isDirectory()) {
                    const cargoPath = path.join(packagePath, 'Cargo.toml');
                    try {
                        await fs.access(cargoPath);
                        packages.push(new DAGNode(
                            entry,
                            vscode.TreeItemCollapsibleState.None,
                            'package',
                            packagePath
                        ));
                    } catch {
                        // No Cargo.toml, skip
                    }
                }
            }
        } catch (error) {
            // Crates directory doesn't exist
        }

        return packages;
    }

    private getIconForNode(element: DAGNode): vscode.ThemeIcon {
        switch (element.type) {
            case 'workspace':
                return new vscode.ThemeIcon('workspace-folder');
            case 'package':
                return new vscode.ThemeIcon('package');
            case 'error':
                return new vscode.ThemeIcon('error');
            default:
                return new vscode.ThemeIcon('file');
        }
    }
}

class DAGNode extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly type: string,
        public readonly path?: string
    ) {
        super(label, collapsibleState);
    }
}