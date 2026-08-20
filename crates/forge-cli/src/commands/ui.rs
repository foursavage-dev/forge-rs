#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;

use forge_core::project::Project;

pub fn run_ui(port: u16, open: bool, project_path: Option<PathBuf>) -> Result<(), anyhow::Error> {
    let root =
        project_path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let bind_addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&bind_addr)?;
    let local_port = listener.local_addr()?.port();
    let url = format!("http://localhost:{}", local_port);

    println!("🦀 Forge Web Dashboard & Telemetry Visualizer");
    println!("============================================================");
    println!("🌐 Dashboard running at: {}", url);
    println!("📂 Workspace root: {}", root.display());
    println!("⚡ Press Ctrl+C to stop dashboard server\n");

    if open {
        let _ = open_browser(&url);
    }

    let root_for_threads = root.clone();
    for stream in listener.incoming().flatten() {
        let root_clone = root_for_threads.clone();
        thread::spawn(move || {
            let _ = handle_http_client(stream, &root_clone);
        });
    }

    Ok(())
}

fn open_browser(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    Ok(())
}

fn handle_http_client(mut stream: TcpStream, root: &Path) -> Result<(), anyhow::Error> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(());
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }

    let path = parts[1];

    if path == "/" || path == "/index.html" {
        let html = generate_dashboard_html(root);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            html.len(),
            html
        );
        stream.write_all(response.as_bytes())?;
        stream.flush()?;
    } else if path == "/api/graph" {
        let json_data = get_workspace_graph_json(root);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
            json_data.len(),
            json_data
        );
        stream.write_all(response.as_bytes())?;
        stream.flush()?;
    } else if path == "/api/stats" {
        let stats_json = get_system_stats_json(root);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
            stats_json.len(),
            stats_json
        );
        stream.write_all(response.as_bytes())?;
        stream.flush()?;
    } else {
        let not_found = "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
        stream.write_all(not_found.as_bytes())?;
        stream.flush()?;
    }

    Ok(())
}

fn get_workspace_graph_json(root: &Path) -> String {
    let mut packages = Vec::new();

    if let Ok(Some(project)) = Project::discover(root) {
        let meta = project.metadata();
        let ws_members: HashSet<_> = meta.workspace_members.iter().collect();

        for pkg in &meta.packages {
            if ws_members.contains(&pkg.id) {
                let internal_deps: Vec<String> = pkg
                    .dependencies
                    .iter()
                    .filter(|d| {
                        meta.packages
                            .iter()
                            .any(|p| p.name == d.name && ws_members.contains(&p.id))
                    })
                    .map(|d| d.name.clone())
                    .collect();

                packages.push(serde_json::json!({
                    "name": pkg.name,
                    "version": pkg.version.to_string(),
                    "dependencies": internal_deps,
                    "type": "rust-crate",
                    "manifest": pkg.manifest_path.to_string()
                }));
            }
        }
    }

    if packages.is_empty() {
        let crates_dir = root.join("crates");
        if let Ok(entries) = fs::read_dir(&crates_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir()
                    && p.join("Cargo.toml").exists()
                    && let Some(name) = p.file_name().and_then(|s| s.to_str())
                {
                    let mut deps = Vec::new();
                    if let Ok(content) = fs::read_to_string(p.join("Cargo.toml")) {
                        for line in content.lines() {
                            if line.starts_with("forge-")
                                && line.contains("path =")
                                && let Some(dep_name) = line.split('=').next()
                            {
                                deps.push(dep_name.trim().to_string());
                            }
                        }
                    }
                    packages.push(serde_json::json!({
                        "name": name,
                        "dependencies": deps,
                        "type": "rust-crate"
                    }));
                }
            }
        }
    }

    if packages.is_empty() {
        packages.push(serde_json::json!({
            "name": "root-project",
            "dependencies": [],
            "type": "root"
        }));
    }

    serde_json::json!({
        "workspace": root.display().to_string(),
        "packages": packages,
        "total": packages.len()
    })
    .to_string()
}

fn get_system_stats_json(_root: &Path) -> String {
    let logical_cores = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    let cache_dir = env::var("FORGE_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs_or_temp_cache());

    let mut total_bytes: u64 = 0;
    let mut records_count = 0;
    let records_dir = cache_dir.join("records");
    if let Ok(entries) = fs::read_dir(&records_dir) {
        for entry in entries.flatten() {
            records_count += 1;
            if let Ok(m) = entry.metadata() {
                total_bytes += m.len();
            }
        }
    }

    let mut objects_count = 0;
    let objects_dir = cache_dir.join("objects");
    if let Ok(entries) = fs::read_dir(&objects_dir) {
        for entry in entries.flatten() {
            objects_count += 1;
            if let Ok(m) = entry.metadata() {
                total_bytes += m.len();
            }
        }
    }

    let cas_dir = cache_dir.join("cas");
    if let Ok(entries) = fs::read_dir(&cas_dir) {
        for entry in entries.flatten() {
            objects_count += 1;
            if let Ok(m) = entry.metadata() {
                total_bytes += m.len();
            }
        }
    }

    let formatted_size = if total_bytes > 1024 * 1024 * 1024 {
        format!("{:.2} GB", total_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if total_bytes > 1024 * 1024 {
        format!("{:.1} MB", total_bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} KB", total_bytes as f64 / 1024.0)
    };

    serde_json::json!({
        "os": os,
        "arch": arch,
        "logical_cores": logical_cores,
        "cache_directory": cache_dir.display().to_string(),
        "cache_records": records_count,
        "cas_objects": objects_count,
        "total_cache_size": formatted_size,
        "engine_version": env!("CARGO_PKG_VERSION")
    })
    .to_string()
}

fn dirs_or_temp_cache() -> PathBuf {
    if let Ok(home) = env::var("USERPROFILE").or_else(|_| env::var("HOME")) {
        PathBuf::from(home).join(".forge").join("cache")
    } else {
        env::temp_dir().join("forge").join("cache")
    }
}

fn generate_dashboard_html(root: &Path) -> String {
    let graph_json = get_workspace_graph_json(root);
    let stats_json = get_system_stats_json(root);
    let engine_version = env!("CARGO_PKG_VERSION");

    let template = r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Forge Telemetry & Interactive DAG Visualizer</title>
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
            --accent-amber: #fbbf24;
            --accent-red: #f87171;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
        body { background-color: var(--bg-base); color: var(--text-primary); min-height: 100vh; display: flex; flex-direction: column; }
        header { background-color: var(--bg-card); border-bottom: 1px solid var(--border-color); padding: 1rem 2rem; display: flex; justify-content: space-between; align-items: center; }
        .brand { display: flex; align-items: center; gap: 0.75rem; font-size: 1.25rem; font-weight: 700; color: var(--text-primary); }
        .brand span { color: var(--accent-blue); }
        .badge { background: rgba(56, 189, 248, 0.15); color: var(--accent-blue); padding: 0.25rem 0.6rem; border-radius: 9999px; font-size: 0.75rem; font-weight: 600; }
        .header-controls { display: flex; align-items: center; gap: 1rem; }
        select.lang-select { background: var(--bg-hover); color: var(--text-primary); border: 1px solid var(--border-color); padding: 0.35rem 0.75rem; border-radius: 6px; font-size: 0.85rem; cursor: pointer; outline: none; }
        main { flex: 1; padding: 2rem; max-width: 1400px; margin: 0 auto; width: 100%; display: flex; flex-direction: column; gap: 2rem; }
        .grid-stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1.25rem; }
        .card { background-color: var(--bg-card); border: 1px solid var(--border-color); border-radius: 12px; padding: 1.5rem; display: flex; flex-direction: column; gap: 0.5rem; }
        .card-title { font-size: 0.85rem; text-transform: uppercase; letter-spacing: 0.05em; color: var(--text-secondary); }
        .card-value { font-size: 1.8rem; font-weight: 700; color: var(--text-primary); }
        .card-meta { font-size: 0.8rem; color: var(--text-secondary); }
        .section { background-color: var(--bg-card); border: 1px solid var(--border-color); border-radius: 12px; padding: 1.5rem; }
        .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.25rem; flex-wrap: wrap; gap: 0.75rem; }
        .section-title { font-size: 1.1rem; font-weight: 600; display: flex; align-items: center; gap: 0.5rem; }
        .graph-controls { display: flex; align-items: center; gap: 0.5rem; }
        .graph-btn { background: var(--bg-hover); color: var(--text-primary); border: 1px solid var(--border-color); padding: 0.35rem 0.75rem; border-radius: 6px; font-size: 0.8rem; cursor: pointer; }
        .graph-btn:hover { background: #374151; }
        .graph-input { background: var(--bg-base); border: 1px solid var(--border-color); color: var(--text-primary); padding: 0.35rem 0.75rem; border-radius: 6px; font-size: 0.85rem; outline: none; }
        .graph-wrapper { position: relative; width: 100%; height: 480px; background: #0c1220; border-radius: 8px; border: 1px solid var(--border-color); overflow: hidden; }
        svg.dag-canvas { width: 100%; height: 100%; cursor: grab; }
        svg.dag-canvas:active { cursor: grabbing; }
        .node-rect { rx: 8; ry: 8; fill: #1f2937; stroke: #4b5563; stroke-width: 1.5; transition: all 0.2s ease; cursor: pointer; }
        .node-rect:hover, .node-rect.selected { stroke: var(--accent-blue); fill: #1e293b; filter: drop-shadow(0 0 8px rgba(56, 189, 248, 0.4)); }
        .node-rect.critical { stroke: var(--accent-amber); }
        .node-text { fill: var(--text-primary); font-size: 12px; font-weight: 600; pointer-events: none; text-anchor: middle; dominant-baseline: central; }
        .edge-line { stroke: #4b5563; stroke-width: 1.5; fill: none; transition: stroke 0.2s; }
        .edge-line.active { stroke: var(--accent-blue); stroke-width: 2.5; }
        .timeline-bar { display: flex; flex-direction: column; gap: 0.5rem; margin-top: 1rem; }
        .timeline-item { display: flex; align-items: center; gap: 1rem; font-size: 0.85rem; }
        .timeline-label { width: 180px; text-align: right; color: var(--text-secondary); text-overflow: ellipsis; overflow: hidden; white-space: nowrap; }
        .timeline-progress { flex: 1; height: 12px; background: var(--bg-hover); border-radius: 6px; overflow: hidden; }
        .timeline-fill { height: 100%; border-radius: 6px; }
    </style>
</head>
<body>
    <header>
        <div class="brand">
            Forge <span>Telemetry</span>
            <span class="badge">v__ENGINE_VERSION__</span>
        </div>
        <div class="header-controls">
            <select class="lang-select" id="langSelect" onchange="changeLanguage(this.value)">
                <option value="en">English (EN)</option>
                <option value="vi">Tiếng Việt (VI)</option>
                <option value="zh-Hans">简体中文 (ZH-CN)</option>
                <option value="zh-Hant">繁體中文 (ZH-TW)</option>
                <option value="ja">日本語 (JA)</option>
            </select>
        </div>
    </header>

    <main>
        <div class="grid-stats">
            <div class="card">
                <div class="card-title" id="lbl-cache-size">Cache Size On Disk</div>
                <div class="card-value" id="stat-cache-size" style="color: var(--accent-green);">--</div>
                <div class="card-meta" id="lbl-cas-meta">Sub-millisecond CAS lookup</div>
            </div>
            <div class="card">
                <div class="card-title" id="lbl-total-pkg">Total Packages</div>
                <div class="card-value" id="stat-pkg-count">--</div>
                <div class="card-meta" id="lbl-dag-nodes">Monorepo DAG Nodes</div>
            </div>
            <div class="card">
                <div class="card-title" id="lbl-cpu-cores">CPU Hardware Cores</div>
                <div class="card-value" id="stat-cores">--</div>
                <div class="card-meta" id="stat-os">Parallel Work-Stealing</div>
            </div>
            <div class="card">
                <div class="card-title" id="lbl-cas-obj">CAS Artifacts</div>
                <div class="card-value" id="stat-cas">--</div>
                <div class="card-meta" id="lbl-dedup-meta">Deduplicated storage</div>
            </div>
        </div>

        <div class="section">
            <div class="section-header">
                <div class="section-title" id="lbl-graph-title">Interactive Dependency Graph (Nx/DAG)</div>
                <div class="graph-controls">
                    <input type="text" class="graph-input" id="nodeSearch" placeholder="Filter packages..." oninput="filterNodes(this.value)" />
                    <button class="graph-btn" onclick="resetZoom()">Reset Zoom</button>
                    <button class="graph-btn" onclick="toggleCriticalPath()" id="btn-crit">Highlight Critical Path</button>
                </div>
            </div>
            <div class="graph-wrapper" id="canvas-wrapper">
                <svg class="dag-canvas" id="dagSvg" viewBox="0 0 1200 480">
                    <defs>
                        <marker id="arrow" viewBox="0 0 10 10" refX="22" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
                            <path d="M 0 0 L 10 5 L 0 10 z" fill="#4b5563"></path>
                        </marker>
                    </defs>
                    <g id="dagGroup"></g>
                </svg>
            </div>
        </div>

        <div class="section">
            <div class="section-header">
                <div class="section-title" id="lbl-timeline-title">DAG Depth & Dependency Weight Distribution</div>
                <span style="font-size: 0.8rem; color: var(--text-secondary);" id="lbl-timeline-sub">Calculated live from workspace graph topology</span>
            </div>
            <div class="timeline-bar" id="timeline-container"></div>
        </div>
    </main>

    <script>
        const i18n = {
            "en": {
                "cache_size": "Cache Size On Disk",
                "cas_meta": "Sub-millisecond CAS lookup",
                "total_pkg": "Total Packages",
                "dag_nodes": "Monorepo DAG Nodes",
                "cpu_cores": "CPU Hardware Cores",
                "cas_obj": "CAS Artifacts",
                "dedup_meta": "Deduplicated storage",
                "graph_title": "Interactive Dependency Graph (Nx/DAG)",
                "filter_placeholder": "Filter packages...",
                "reset_zoom": "Reset Zoom",
                "highlight_crit": "Highlight Critical Path",
                "timeline_title": "DAG Depth & Dependency Weight Distribution",
                "timeline_sub": "Calculated live from workspace graph topology"
            },
            "vi": {
                "cache_size": "Dung lượng Cache Trên Ổ đĩa",
                "cas_meta": "Truy xuất CAS dưới 1 mili-giây",
                "total_pkg": "Tổng số Package",
                "dag_nodes": "Các nút Đồ thị Monorepo",
                "cpu_cores": "Số nhân CPU Phần cứng",
                "cas_obj": "Dữ liệu Artifacts CAS",
                "dedup_meta": "Bộ nhớ khử trùng lặp",
                "graph_title": "Đồ thị Phụ thuộc Trực quan (Nx/DAG)",
                "filter_placeholder": "Lọc package...",
                "reset_zoom": "Đặt lại Góc nhìn",
                "highlight_crit": "Làm nổi bật Đường găng",
                "timeline_title": "Độ sâu DAG & Phân phối Trọng số Phụ thuộc",
                "timeline_sub": "Tính toán trực tiếp từ cấu trúc đồ thị thực tế"
            },
            "zh-Hans": {
                "cache_size": "磁盘缓存总容量",
                "cas_meta": "亚毫秒级 CAS 检索",
                "total_pkg": "软件包总数",
                "dag_nodes": "Monorepo DAG 节点",
                "cpu_cores": "CPU 硬件核心数",
                "cas_obj": "CAS 构件数量",
                "dedup_meta": "去重持久化存储",
                "graph_title": "交互式依赖关系拓扑图 (DAG)",
                "filter_placeholder": "过滤软件包...",
                "reset_zoom": "重置缩放",
                "highlight_crit": "高亮关键路径",
                "timeline_title": "DAG 深度与依赖权重分布",
                "timeline_sub": "根据工作区拓扑实时计算"
            },
            "zh-Hant": {
                "cache_size": "磁碟快取總容量",
                "cas_meta": "亞毫秒級 CAS 檢索",
                "total_pkg": "軟體包總數",
                "dag_nodes": "Monorepo DAG 節點",
                "cpu_cores": "CPU 硬體核心數",
                "cas_obj": "CAS 構件數量",
                "dedup_meta": "去重持久化儲存",
                "graph_title": "互動式依賴關係拓撲圖 (DAG)",
                "filter_placeholder": "過濾軟體包...",
                "reset_zoom": "重設縮放",
                "highlight_crit": "高亮關鍵路徑",
                "timeline_title": "DAG 深度與依賴權重分佈",
                "timeline_sub": "根據工作區拓撲即時計算"
            },
            "ja": {
                "cache_size": "ディスクキャッシュサイズ",
                "cas_meta": "ミリ秒未満のCASルックアップ",
                "total_pkg": "合計パッケージ数",
                "dag_nodes": "モノレポDAGノード",
                "cpu_cores": "CPUハードウェアコア",
                "cas_obj": "CASアーティファクト",
                "dedup_meta": "重複排除ストレージ",
                "graph_title": "インタラクティブ依存関係グラフ (DAG)",
                "filter_placeholder": "パッケージを検索...",
                "reset_zoom": "ズームをリセット",
                "highlight_crit": "クリティカルパス強調",
                "timeline_title": "DAG深度と依存関係重み分布",
                "timeline_sub": "ワークスペース構造からリアルタイム計算"
            }
        };

        function changeLanguage(lang) {
            const dict = i18n[lang] || i18n['en'];
            document.getElementById('lbl-cache-size').innerText = dict.cache_size;
            document.getElementById('lbl-cas-meta').innerText = dict.cas_meta;
            document.getElementById('lbl-total-pkg').innerText = dict.total_pkg;
            document.getElementById('lbl-dag-nodes').innerText = dict.dag_nodes;
            document.getElementById('lbl-cpu-cores').innerText = dict.cpu_cores;
            document.getElementById('lbl-cas-obj').innerText = dict.cas_obj;
            document.getElementById('lbl-dedup-meta').innerText = dict.dedup_meta;
            document.getElementById('lbl-graph-title').innerText = dict.graph_title;
            document.getElementById('nodeSearch').placeholder = dict.filter_placeholder;
            document.getElementById('btn-crit').innerText = dict.highlight_crit;
            document.getElementById('lbl-timeline-title').innerText = dict.timeline_title;
            document.getElementById('lbl-timeline-sub').innerText = dict.timeline_sub;
        }

        const graphData = __GRAPH_JSON__;
        const statsData = __STATS_JSON__;

        document.getElementById('stat-pkg-count').innerText = graphData.total || graphData.packages.length;
        document.getElementById('stat-cores').innerText = statsData.logical_cores;
        document.getElementById('stat-os').innerText = `${statsData.os} (${statsData.arch})`;
        document.getElementById('stat-cas').innerText = (statsData.cas_objects || 0) + (statsData.cache_records || 0);
        document.getElementById('stat-cache-size').innerText = statsData.total_cache_size || "0.0 KB";

        const dagGroup = document.getElementById('dagGroup');
        const nodeWidth = 150;
        const nodeHeight = 36;
        let positions = {};
        let isCriticalActive = false;

        function renderDag() {
            dagGroup.innerHTML = '';
            const cols = 5;
            const xSpacing = 230;
            const ySpacing = 80;
            
            graphData.packages.forEach((pkg, idx) => {
                const col = idx % cols;
                const row = Math.floor(idx / cols);
                const x = 40 + col * xSpacing;
                const y = 30 + row * ySpacing;
                positions[pkg.name] = { x, y };
            });

            graphData.packages.forEach(pkg => {
                const fromPos = positions[pkg.name];
                if (!fromPos) return;
                (pkg.dependencies || []).forEach(dep => {
                    const toPos = positions[dep];
                    if (toPos) {
                        const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
                        line.setAttribute('x1', fromPos.x + nodeWidth);
                        line.setAttribute('y1', fromPos.y + nodeHeight / 2);
                        line.setAttribute('x2', toPos.x);
                        line.setAttribute('y2', toPos.y + nodeHeight / 2);
                        line.setAttribute('class', 'edge-line');
                        line.setAttribute('marker-end', 'url(#arrow)');
                        line.dataset.from = pkg.name;
                        line.dataset.to = dep;
                        dagGroup.appendChild(line);
                    }
                });
            });

            graphData.packages.forEach(pkg => {
                const pos = positions[pkg.name];
                if (!pos) return;
                const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
                g.dataset.name = pkg.name;

                const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
                rect.setAttribute('x', pos.x);
                rect.setAttribute('y', pos.y);
                rect.setAttribute('width', nodeWidth);
                rect.setAttribute('height', nodeHeight);
                rect.setAttribute('class', 'node-rect');
                rect.onclick = () => focusNode(pkg.name);

                const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
                text.setAttribute('x', pos.x + nodeWidth / 2);
                text.setAttribute('y', pos.y + nodeHeight / 2);
                text.setAttribute('class', 'node-text');
                text.textContent = pkg.name;

                g.appendChild(rect);
                g.appendChild(text);
                dagGroup.appendChild(g);
            });
        }

        function focusNode(name) {
            document.querySelectorAll('.node-rect').forEach(el => el.classList.remove('selected'));
            document.querySelectorAll('.edge-line').forEach(el => el.classList.remove('active'));
            const nodeG = document.querySelector(`g[data-name="${name}"] rect`);
            if (nodeG) nodeG.classList.add('selected');
            document.querySelectorAll(`line[data-from="${name}"], line[data-to="${name}"]`).forEach(l => l.classList.add('active'));
        }

        function filterNodes(query) {
            const q = query.toLowerCase().trim();
            document.querySelectorAll('#dagGroup g').forEach(g => {
                const name = g.dataset.name.toLowerCase();
                g.style.opacity = (!q || name.includes(q)) ? '1' : '0.15';
            });
        }

        function toggleCriticalPath() {
            isCriticalActive = !isCriticalActive;
            document.querySelectorAll('.node-rect').forEach((r) => {
                const name = r.parentElement.dataset.name;
                const pkg = graphData.packages.find(p => p.name === name);
                const hasDeps = pkg && pkg.dependencies && pkg.dependencies.length > 2;
                if (isCriticalActive && hasDeps) {
                    r.classList.add('critical');
                } else {
                    r.classList.remove('critical');
                }
            });
        }

        function resetZoom() {
            document.getElementById('dagSvg').setAttribute('viewBox', '0 0 1200 480');
        }

        renderDag();

        const timelineContainer = document.getElementById('timeline-container');
        timelineContainer.innerHTML = '';
        const colors = ['#38bdf8', '#34d399', '#a78bfa', '#fbbf24', '#f472b6'];

        const sortedPkgs = [...graphData.packages].sort((a, b) => {
            const depA = (a.dependencies || []).length;
            const depB = (b.dependencies || []).length;
            return depB - depA;
        });

        const maxDeps = Math.max(...sortedPkgs.map(p => (p.dependencies || []).length), 1);

        sortedPkgs.slice(0, 10).forEach((pkg, idx) => {
            const depCount = (pkg.dependencies || []).length;
            const pct = Math.max(Math.round((depCount / maxDeps) * 90) + 10, 15);
            const color = colors[idx % colors.length];
            const row = document.createElement('div');
            row.className = 'timeline-item';
            row.innerHTML = `
                <div class="timeline-label" title="${pkg.name}">${pkg.name}</div>
                <div class="timeline-progress">
                    <div class="timeline-fill" style="width: ${pct}%; background: ${color};"></div>
                </div>
                <div style="width: 100px; font-size: 0.75rem; color: var(--text-secondary); text-align: right;">${depCount} direct deps</div>
            `;
            timelineContainer.appendChild(row);
        });
    </script>
</body>
</html>"##;

    template
        .replace("__ENGINE_VERSION__", engine_version)
        .replace("__GRAPH_JSON__", &graph_json)
        .replace("__STATS_JSON__", &stats_json)
}
