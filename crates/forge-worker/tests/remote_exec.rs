use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use forge_executor::{CommandSpec, Task, TaskExecutor, TaskStatus};
use forge_worker::{RemoteWorkerClient, WorkerServer};

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
