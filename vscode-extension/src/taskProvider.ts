import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs/promises';

export class FishTaskProvider implements vscode.TreeDataProvider<TaskNode> {
    private _onDidChangeTreeData = new vscode.EventEmitter<TaskNode | undefined | null | void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    constructor(
        private context: vscode.ExtensionContext,
        private fishExecutable: string
    ) {}

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: TaskNode): vscode.TreeItem {
        const treeItem = new vscode.TreeItem(element.label, element.collapsibleState);
        treeItem.iconPath = this.getIconForTask(element);
        treeItem.contextValue = element.type;
        if (element.commandId) {
            treeItem.command = {
                command: element.commandId,
                title: element.label,
                arguments: [element]
            };
        }
        return treeItem;
    }

    async getChildren(element?: TaskNode): Promise<TaskNode[]> {
        if (!element) {
            return this.getTaskCategories();
        }
        return this.getTasksForCategory(element);
    }

    private async getTaskCategories(): Promise<TaskNode[]> {
        return [
            new TaskNode('Build Tasks', vscode.TreeItemCollapsibleState.Collapsed, 'category', 'build'),
            new TaskNode('Test Tasks', vscode.TreeItemCollapsibleState.Collapsed, 'category', 'test'),
            new TaskNode('Maintenance', vscode.TreeItemCollapsibleState.Collapsed, 'category', 'maintenance')
        ];
    }

    private async getTasksForCategory(category: TaskNode): Promise<TaskNode[]> {
        switch (category.category) {
            case 'build':
                return [
                    new TaskNode('Full Build', vscode.TreeItemCollapsibleState.None, 'task', 'build', 'fish.runBuild'),
                    new TaskNode('Check Only', vscode.TreeItemCollapsibleState.None, 'task', 'check', 'fish.runBuild'),
                    new TaskNode('Incremental Build', vscode.TreeItemCollapsibleState.None, 'task', 'incremental', 'fish.runBuild')
                ];
            case 'test':
                return [
                    new TaskNode('Run All Tests', vscode.TreeItemCollapsibleState.None, 'task', 'test', 'fish.runTest'),
                    new TaskNode('Test with Coverage', vscode.TreeItemCollapsibleState.None, 'task', 'coverage', 'fish.runTest'),
                    new TaskNode('Run Specific Test', vscode.TreeItemCollapsibleState.None, 'task', 'specific', 'fish.runTest')
                ];
            case 'maintenance':
                return [
                    new TaskNode('Clean Build', vscode.TreeItemCollapsibleState.None, 'task', 'clean', 'fish.clean'),
                    new TaskNode('Refresh Graph', vscode.TreeItemCollapsibleState.None, 'task', 'refresh', 'fish.refreshGraph'),
                    new TaskNode('Generate Diagnostics', vscode.TreeItemCollapsibleState.None, 'task', 'diagnostics', 'fish.showDiagnostics')
                ];
            default:
                return [];
        }
    }

    private getIconForTask(element: TaskNode): vscode.ThemeIcon {
        switch (element.type) {
            case 'category':
                return new vscode.ThemeIcon('folder');
            case 'task':
                return new vscode.ThemeIcon('play-circle');
            default:
                return new vscode.ThemeIcon('file');
        }
    }
}

class TaskNode extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly type: string,
        public readonly category?: string,
        public readonly commandId?: string
    ) {
        super(label, collapsibleState);
    }
}