use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::config::{Cmd, Script};

struct DeleteFileHandle(String);

impl Drop for DeleteFileHandle {
    fn drop(&mut self) {
        fs::remove_file(&self.0).unwrap();
    }
}

// creates temporary .sh file to run your script
fn execute(cmd: &str) -> (Command, DeleteFileHandle) {
    let fname = format!(
        "~{}.sh",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    );

    fs::write(&fname, cmd).unwrap();

    let mut command = Command::new("sh");
    command.arg(&fname);

    (command, DeleteFileHandle(fname))
}

fn run_cmd(cmd: &Cmd) {
    match cmd {
        Cmd::Cmd(cmd) => {
            eprintln!("$ {}", cmd);

            let (mut command, _handle) = execute(cmd);
            command.spawn().unwrap().wait().unwrap();
        }
        Cmd::Consecutive(cmds) => {
            for cmd in cmds {
                run_cmd(cmd);
            }
        }
    }
}

pub fn run_script(script: &Script) {
    eprintln!("\u{001b}[32mRunning script\u{001b}[0m {}", script.name);
    run_cmd(&script.cmd);
}
