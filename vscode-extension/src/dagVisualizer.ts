import * as vscode from 'vscode';
import * as path from 'path';

export class FishDAGVisualizer {
    private panel: vscode.WebviewPanel | undefined;
    private context: vscode.ExtensionContext;
    private fishExecutable: string;

    constructor(context: vscode.ExtensionContext, fishExecutable: string) {
        this.context = context;
        this.fishExecutable = fishExecutable;
    }

    show() {
        if (this.panel) {
            this.panel.reveal();
            return;
        }

        this.panel = vscode.window.createWebviewPanel(
            'fishDAG',
            'Fish Dependency Graph',
            vscode.ViewColumn.One,
            {
                enableScripts: true,
                retainContextWhenHidden: true
            }
        );

        this.panel.webview.html = this.getWebviewContent();
        this.panel.onDidDispose(() => {
            this.panel = undefined;
        });

        // Setup message handler
        this.panel.webview.onDidReceiveMessage(
            message => {
                switch (message.command) {
                    case 'getGraphData':
                        this.sendGraphData();
                        break;
                    case 'refresh':
                        this.refreshGraph();
                        break;
                }
            }
        );
    }

    private getWebviewContent(): string {
        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Fish Dependency Graph</title>
    <style>
        :root {
            --bg-base: #090d16;
            --bg-card: #111827;
            --bg-hover: #1f2937;
            --border-color: #374151;
            --text-primary: #f9fafb;
            --text-secondary: #9ca3af;
            --accent-blue: #38bdf8;
            --accent-green: #34d399;
            --accent-purple: #a78bfa;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
        body { background-color: var(--bg-base); color: var(--text-primary); padding: 20px; }
        .header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
        .title { font-size: 1.5rem; font-weight: 700; color: var(--accent-blue); }
        .controls { display: flex; gap: 10px; }
        button { background: var(--bg-hover); color: var(--text-primary); border: 1px solid var(--border-color); padding: 8px 16px; border-radius: 6px; cursor: pointer; }
        button:hover { background: #374151; }
        .graph-container { width: 100%; height: 600px; background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 12px; overflow: hidden; }
        #dagSvg { width: 100%; height: 100%; }
        .node-rect { rx: 8; ry: 8; fill: #1f2937; stroke: #4b5563; stroke-width: 1.5; cursor: pointer; transition: all 0.2s; }
        .node-rect:hover { stroke: var(--accent-blue); fill: #1e293b; }
        .node-text { fill: var(--text-primary); font-size: 12px; font-weight: 600; pointer-events: none; text-anchor: middle; dominant-baseline: central; }
        .edge-line { stroke: #4b5563; stroke-width: 1.5; fill: none; }
        .stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin-top: 20px; }
        .stat-card { background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 8px; padding: 15px; }
        .stat-label { font-size: 0.85rem; color: var(--text-secondary); margin-bottom: 5px; }
        .stat-value { font-size: 1.5rem; font-weight: 700; color: var(--accent-green); }
    </style>
</head>
<body>
    <div class="header">
        <div class="title">🐟 Fish Dependency Graph</div>
        <div class="controls">
            <button onclick="refreshGraph()">🔄 Refresh</button>
            <button onclick="resetZoom()">🔍 Reset Zoom</button>
        </div>
    </div>
    
    <div class="graph-container">
        <svg id="dagSvg" viewBox="0 0 1200 600">
            <defs>
                <marker id="arrow" viewBox="0 0 10 10" refX="22" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
                    <path d="M 0 0 L 10 5 L 0 10 z" fill="#4b5563"></path>
                </marker>
            </defs>
            <g id="dagGroup"></g>
        </svg>
    </div>

    <div class="stats">
        <div class="stat-card">
            <div class="stat-label">Total Packages</div>
            <div class="stat-value" id="statPackages">--</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Dependencies</div>
            <div class="stat-value" id="statDeps">--</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Critical Path</div>
            <div class="stat-value" id="statCritical">--</div>
        </div>
    </div>

    <script>
        const vscode = acquireVsCodeApi();
        
        function refreshGraph() {
            vscode.postMessage({ command: 'refresh' });
        }
        
        function resetZoom() {
            // Reset zoom implementation
        }

        // Request initial graph data
        vscode.postMessage({ command: 'getGraphData' });

        vscode.postMessage({ command: 'getGraphData' });

        // Handle messages from extension
        window.addEventListener('message', event => {
            const message = event.data;
            if (message.type === 'graphData') {
                renderGraph(message.data);
            }
        });

        function renderGraph(graphData) {
            const svg = document.getElementById('dagGroup');
            svg.innerHTML = '';
            
            const nodes = graphData.packages || [];
            const nodeWidth = 120;
            const nodeHeight = 40;
            const horizontalGap = 50;
            const verticalGap = 30;
            
            // Simple layout: arrange nodes in layers
            const levels = {};
            nodes.forEach((node, index) => {
                const level = node.dependencies.length;
                if (!levels[level]) levels[level] = [];
                levels[level].push({ ...node, index });
            });

            let yOffset = 50;
            Object.keys(levels).forEach(level => {
                const levelNodes = levels[level];
                const totalWidth = levelNodes.length * (nodeWidth + horizontalGap) - horizontalGap;
                let xOffset = (1200 - totalWidth) / 2;

                levelNodes.forEach(node => {
                    const x = xOffset;
                    const y = yOffset;
                    
                    // Draw node
                    const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
                    rect.setAttribute('x', x);
                    rect.setAttribute('y', y);
                    rect.setAttribute('width', nodeWidth);
                    rect.setAttribute('height', nodeHeight);
                    rect.setAttribute('class', 'node-rect');
                    svg.appendChild(rect);

                    // Draw label
                    const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
                    text.setAttribute('x', x + nodeWidth / 2);
                    text.setAttribute('y', y + nodeHeight / 2);
                    text.setAttribute('class', 'node-text');
                    text.textContent = node.name.substring(0, 10);
                    svg.appendChild(text);

                    // Draw edges
                    node.dependencies.forEach(depName => {
                        const depNode = nodes.find(n => n.name === depName);
                        if (depNode) {
                            const edge = document.createElementNS('http://www.w3.org/2000/svg', 'line');
                            edge.setAttribute('x1', x + nodeWidth);
                            edge.setAttribute('y1', y + nodeHeight / 2);
                            edge.setAttribute('x2', x + nodeWidth + horizontalGap / 2);
                            edge.setAttribute('y2', y + nodeHeight / 2);
                            edge.setAttribute('class', 'edge-line');
                            edge.setAttribute('marker-end', 'url(#arrow)');
                            svg.appendChild(edge);
                        }
                    });

                    xOffset += nodeWidth + horizontalGap;
                });

                yOffset += nodeHeight + verticalGap;
            });

            // Update stats
            document.getElementById('statPackages').textContent = nodes.length;
            document.getElementById('statDeps').textContent = nodes.reduce((sum, node) => sum + node.dependencies.length, 0);
            document.getElementById('statCritical').textContent = Object.keys(levels).length;
        }
    </script>
</body>
</html>`;
    }

    private async sendGraphData() {
        if (!this.panel) return;

        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders) {
            this.panel.webview.postMessage({ type: 'graphData', data: { packages: [] } });
            return;
        }

        try {
            // For now, use basic package discovery
            const packages = await this.discoverPackages(workspaceFolders[0].uri.fsPath);
            this.panel.webview.postMessage({ type: 'graphData', data: { packages } });
        } catch (error) {
            console.error('Error getting graph data:', error);
            this.panel.webview.postMessage({ type: 'graphData', data: { packages: [] } });
        }
    }

    private async discoverPackages(workspacePath: string): Promise<any[]> {
        const packages: any[] = [];
        const cratesPath = path.join(workspacePath, 'crates');
        
        try {
            const fs = require('fs/promises');
            const entries = await fs.readdir(cratesPath);
            
            for (const entry of entries) {
                const packagePath = path.join(cratesPath, entry);
                const stat = await fs.stat(packagePath);
                
                if (stat.isDirectory()) {
                    const cargoPath = path.join(packagePath, 'Cargo.toml');
                    try {
                        await fs.access(cargoPath);
                        const cargoContent = await fs.readFile(cargoPath, 'utf-8');
                        const dependencies = this.parseDependencies(cargoContent);
                        
                        packages.push({
                            name: entry,
                            dependencies: dependencies,
                            type: 'rust-crate'
                        });
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

    private parseDependencies(cargoContent: string): string[] {
        const dependencies: string[] = [];
        const lines = cargoContent.split('\n');
        
        for (const line of lines) {
            if (line.includes('fish-') && line.includes('path =')) {
                const match = line.match(/fish-[\w-]+/);
                if (match) {
                    dependencies.push(match[0]);
                }
            }
        }
        
        return dependencies;
    }

    private refreshGraph() {
        this.sendGraphData();
    }
}