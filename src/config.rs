use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Script {
    pub name: String,
    pub cmd: String,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Config {
    pub scripts: Vec<Script>,
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! setup_test {
        ($test_name:ident, $pstr:expr, $result:expr) => {
            #[test]
            fn $test_name() {
                let s = $pstr;

                let sconfig = toml::from_str(s);

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
            scripts: vec![Script {
                name: String::from("run"),
                cmd: String::from("echo run"),
            }],
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
            scripts: vec![
                Script{
                    name: String::from("run"),
                    cmd: String::from("echo run")
                },
                Script {
                    name: String::from("backrun"),
                    cmd: String::from("echo backrun"),
                }
            ]
        })
    }
}
