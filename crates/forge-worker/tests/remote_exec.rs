use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use forge_executor::{CommandSpec, ProcessExecutor, Task, TaskExecutor, TaskStatus};
use forge_worker::{ClusterExecutor, RemoteWorkerClient, WorkerServer};

#[test]
fn remote_worker_executes_task_over_tcp() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let _ = WorkerServer::handle_client(&mut stream, &Some("secret-token".to_string()));
        }
    });

    thread::sleep(Duration::from_millis(50));

    let client = RemoteWorkerClient::new(&addr, Some("secret-token".to_string()));
    let spec = CommandSpec::new("node").args(["-e", "console.log('remote ok')"]);
    let task = Task::new("remote_sample", spec.command_line(), spec);

    let outcome = client.execute(&task).unwrap();
    assert_eq!(outcome.status, TaskStatus::Executed);
    assert!(outcome.stdout.contains("remote ok"));
}

#[test]
fn remote_worker_ping_and_health() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    drop(listener);

    let server = WorkerServer::with_options(
        &addr,
        Some("auth_token_xyz".to_string()),
        "test-worker-1",
        4,
    );
    let _handle = server.start_background().unwrap();
    thread::sleep(Duration::from_millis(50));

    let client = RemoteWorkerClient::new(&addr, Some("auth_token_xyz".to_string()));
    let ping = client.ping().unwrap();
    assert_eq!(ping.status, "ok");
    assert_eq!(ping.health.worker_name, "test-worker-1");
    assert_eq!(ping.health.max_concurrency, 4);

    server.stop();
}

#[test]
fn cluster_executor_failover_to_local() {
    let dead_addr = "127.0.0.1:59999";
    let dead_client = RemoteWorkerClient::new(dead_addr, None);

    let local = Arc::new(ProcessExecutor::default());
    let cluster = ClusterExecutor::with_local_fallback(vec![dead_client], local);

    let spec = CommandSpec::new("node").args(["-e", "console.log('failover worked')"]);
    let task = Task::new("failover_test", spec.command_line(), spec);

    let outcome = cluster.execute(&task).unwrap();
    assert_eq!(outcome.status, TaskStatus::Executed);
    assert!(outcome.stdout.contains("failover worked"));
}

#[test]
fn source_snapshot_is_shipped_to_and_used_by_the_worker() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let _ = WorkerServer::handle_client(&mut stream, &None);
        }
    });
    thread::sleep(Duration::from_millis(50));

    let src = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(src.path().join("target")).unwrap();
    std::fs::write(src.path().join("payload.txt"), "snapshot me").unwrap();
    std::fs::write(src.path().join("target/garbage.bin"), b"x").unwrap();

    let client = RemoteWorkerClient::new(&addr, None).with_source_packaging();
    let script = "const fs=require('fs'); if (fs.existsSync('target/garbage.bin')) { process.exit(2); } console.log(fs.readFileSync('payload.txt', 'utf8').length)";
    let spec = CommandSpec::new("node")
        .args(["-e", script])
        .cwd(src.path().to_path_buf());
    let task = Task::new("source_snapshot", spec.command_line(), spec);

    let outcome = client.execute(&task).unwrap();
    assert_eq!(outcome.status, TaskStatus::Executed);
    assert_eq!(outcome.stdout.trim(), "11");
}

#[test]
fn cluster_source_packaging_is_forwarded_to_workers() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let _ = WorkerServer::handle_client(&mut stream, &None);
        }
    });
    thread::sleep(Duration::from_millis(50));

    let worker = RemoteWorkerClient::new(&addr, None);
    let cluster = ClusterExecutor::without_fallback(vec![worker]).with_source_packaging();

    let src = tempfile::tempdir().unwrap();
    std::fs::write(src.path().join("data.txt"), "cluster me").unwrap();
    let spec = CommandSpec::new("node")
        .args(["-e", "console.log(require('fs').readFileSync('data.txt', 'utf8').length)"])
        .cwd(src.path().to_path_buf());
    let task = Task::new("cluster_snapshot", spec.command_line(), spec);

    let outcome = cluster.execute(&task).unwrap();
    assert_eq!(outcome.status, TaskStatus::Executed);
    assert_eq!(outcome.stdout.trim(), "10", "stderr: {:?}", outcome.stderr);
}
