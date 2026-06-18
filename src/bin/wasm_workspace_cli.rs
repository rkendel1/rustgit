use rustgit_wasm_runtime::{WasmWorkspace, WorkspaceManager};

fn main() {
    let mut args = std::env::args();
    let _binary = args.next();

    let command = match args.next() {
        Some(c) => c,
        None => {
            eprintln!("usage: wasm-workspace-cli <launch|stop|restart|logs> <arg>");
            return;
        }
    };

    let manager = WorkspaceManager::new(".wasm-runtime");

    match command.as_str() {
        "launch" => {
            let repo = args.next().unwrap_or_default();
            match manager.launch(&repo) {
                Ok(workspace) => {
                    println!(
                        "launched workspace {} for {}",
                        workspace.id, workspace.repo_url
                    );
                }
                Err(err) => eprintln!("launch failed: {err}"),
            }
        }
        "stop" => {
            let id = args.next().unwrap_or_default();
            if let Err(err) = manager.stop(&id) {
                eprintln!("stop failed: {err}");
            }
        }
        "restart" => {
            let id = args.next().unwrap_or_default();
            if let Err(err) = manager.restart(&id) {
                eprintln!("restart failed: {err}");
            }
        }
        "logs" => {
            let id = args.next().unwrap_or_default();
            match manager.logs(&id) {
                Ok(lines) => {
                    for line in lines {
                        println!("{line}");
                    }
                }
                Err(err) => eprintln!("logs failed: {err}"),
            }
        }
        _ => eprintln!("unknown command: {command}"),
    }
}
