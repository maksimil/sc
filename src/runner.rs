use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::config::{Cmd, Script};

fn write_commands(buf: &mut String, cmd: &Cmd) {
    match cmd {
        Cmd::String(s) => {
            *buf += &format!("echo \u{001b}[32m$\u{001b}[0m {}\n", s);
            *buf += &format!("{}\n", s);
        }
        Cmd::Consecutive(cmds) => {
            for cmd in cmds {
                write_commands(buf, cmd);
            }
        }
    }
}

pub fn run_script(script: &Script) {
    // converting script to sh format
    let sh = {
        let mut s = String::new();
        if let Some(cwd) = &script.cwd {
            s += &format!("cd {}\n", cwd);
        }

        write_commands(&mut s, &script.cmd);

        s
    };

    // create a temporary .sh file and write the script to
    let (fname, mut command) = {
        let fname = format!(
            "~{}.sh",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        );

        fs::write(&fname, sh).unwrap();

        let mut command = Command::new("sh");
        command.arg(&fname);
        (fname, command)
    };

    // run the script
    command.spawn().unwrap().wait().unwrap();

    // remove the temp file
    fs::remove_file(fname).unwrap();
}
