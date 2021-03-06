use config::Env;

use super::*;

macro_rules! setup_test {
    ($test_name:ident, $pstr:expr, $result:expr) => {
        #[test]
        fn $test_name() {
            let s = $pstr;

            let sconfig = s.parse::<Config>();

            let config = $result;

            assert_eq!(sconfig, config);
        }
    };
}

setup_test! {
    regular_parse,
    "
    [[scripts]]
    name = \"run\"
    cmd = \"echo run\"
    ",
    Ok(Config {
        config: Env(vec![]),
        scripts: vec![
            (
                String::from("run"),
                Script {
                    cmd: Cmd::String(String::from("echo run")),
                    cwd: None,
                    env: Env(vec![])
                }
            )
        ].into_iter().collect(),
    })
}

setup_test! {
    multiple_entries_parse,
    "
    [[scripts]]
    name = \"run\"
    cmd = \"echo run\"

    [[scripts]]
    name = \"backrun\"
    cmd = \"echo backrun\"
    ",
    Ok(Config {
        config: Env(vec![]),
        scripts: vec![
            (
                String::from("run"),
                Script{
                    cmd: Cmd::String(String::from("echo run")),
                    cwd: None,
                    env: Env(vec![])
                }
            ),
            (
                String::from("backrun"),
                Script {
                    cmd: Cmd::String(String::from("echo backrun")),
                    cwd: None,
                    env: Env(vec![])
                }
            )
        ].into_iter().collect()
    })
}

setup_test! {
    simple_toml_error,
    "
    [[scripts]]
    name = \"run\"
    cmd = 
    ",
    Err(String::from("expected a value, found a newline at line 4 column 11"))
}

setup_test! {
    config_errors_1,
    "
    [[scripts]]
    name = \"ru n\"
    cmd = \"echo run\"
    ",
    Err(String::from("script 0:ru n : script name cannot contain tabs or spaces"))
}

setup_test! {
    config_errors_2,
    "
    [[scripts]]
    name = \"r\"
    cmd = \"echo e\"

    [[scripts]]
    name = \"\"
    cmd = \"echo run\"
    ",
    Err(String::from("script 1: script name cannot be empty"))
}

setup_test! {
    config_errors_3,
    "
    [[scripts]]
    name = \"-b\"
    cmd = \"echo run\"
    ",
    Err(String::from("script 0:-b : script name cannot start with -"))
}

setup_test! {
    sequenced_commands,
    "
    [[scripts]]
    name = \"s\"
    cmd = [\"echo a\", \"echo b\"]
    
    [[scripts]]
    name = \"alpha\"
    cmd = \"echo beta\"
    ",
    Ok(Config {
        config: Env(vec![]),
        scripts: vec![
            (
                String::from("s"),
                Script {
                    cmd: Cmd::Consecutive(vec![Cmd::String(String::from("echo a")),Cmd::String(String::from("echo b"))]),
                    cwd: None,
                    env: Env(vec![])
                }
            ),
            (
                String::from("alpha"),
                Script {
                    cmd: Cmd::String(String::from("echo beta")),
                    cwd: None,
                    env: Env(vec![])
                }
            )
        ].into_iter().collect(),
    })
}

setup_test! {
    set_cwd,
    "
    [[scripts]]
    name = \"hey\"
    cwd = \"./target\"
    cmd = [\"echo hey\", \"touch d.txt\"]
    ",
    Ok(Config {
        config: Env(vec![]),
        scripts: vec![
            (
                String::from("hey"),
                Script {
                    cmd: Cmd::Consecutive(vec![Cmd::String(String::from("echo hey")), Cmd::String(String::from("touch d.txt"))]),
                    cwd: Some(String::from("./target")),
                    env: Env(vec![])
                }
            )
        ].into_iter().collect()
    })
}

setup_test! {
    name_conflict,
    "
    [[scripts]]
    name = \"ab\"
    cmd = \"echo ab\"

    [[scripts]]
    name = \"ab\"
    cmd = \"echo ba\"
    ",
    Err(String::from("2 scripts have the same name \"ab\""))
}

setup_test! {
    set_env,
    "
    [[scripts]]
    name = \"env_a\"
    cmd = \"echo $NAME\"
    env = {NAME=\"hey\", \"SUR_NAME\"=\"bye\"}
    ",
    Ok(Config {
        config: Env(vec![]),
        scripts: vec![
            (
                String::from("env_a"),
                Script {
                    cmd: Cmd::String(String::from("echo $NAME")),
                    cwd: None,
                    env: Env(vec![String::from("NAME=hey"), String::from("SUR_NAME=bye")]),
                }
            )
        ].into_iter().collect()
    })
}

setup_test! {
    invalid_env_name,
    "
    [[scripts]]
    name = \"env_a\"
    cmd = \"echo $NAME\"
    env = {\"hey you\"=\"hey\"}
    ",
    Err(String::from("script 0:env_a : invalid env variable name \"hey you\""))
}

setup_test! {
    set_global_config,
    "
    [config]
    NAME = \"GLOBAL\"

    [[scripts]]
    name = \"env_a\"
    cmd = \"echo $NAME\"
    ",
    Ok(Config {
        config: Env(vec![String::from("NAME=GLOBAL")]),
        scripts: vec![
            (
                String::from("env_a"),
                Script {
                    cmd: Cmd::String(String::from("echo $NAME")),
                    cwd: None,
                    env: Env(vec![])
                }
            )
        ].into_iter().collect()
    })
}

setup_test! {
    global_config_err,
    "
    [config]
    \"NAME A\" = \"GLOBAL\"

    [[scripts]]
    name = \"env_a\"
    cmd = \"echo $NAME\"
    ",
    Err(String::from("config : invalid env variable name \"NAME A\""))
}
