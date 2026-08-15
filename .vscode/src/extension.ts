import * as vscode from 'vscode';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export function activate(context: vscode.ExtensionContext) {
    console.log('Forge extension is now active!');

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

    context.subscriptions.push(
        buildCommand,
        testCommand,
        graphCommand,
        cleanCommand,
        doctorCommand,
        affectedCommand
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

        const terminal = vscode.window.createTerminal(`Forge ${command}`);
        terminal.sendText(`cd "${workspaceFolder.uri.fsPath}"`);
        terminal.sendText(cmd);
        terminal.show();
    } catch (error) {
        vscode.window.showErrorMessage(`Forge command failed: ${error}`);
    }
}

export function deactivate() {
    console.log('Forge extension is now deactivated');
}