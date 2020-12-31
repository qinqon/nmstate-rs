use clap;
use nmstate::NetState;
use serde_yaml;

const SUB_CMD_GEN_CONF: &str = "gc";

struct CliError {
    msg: String,
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

fn main() {
    let matches = clap::App::new("nmstatectl")
        .version("1.0")
        .author("Gris Ge <fge@redhat.com>")
        .about("Command line of nmstate")
        .setting(clap::AppSettings::SubcommandRequired)
        .subcommand(
            clap::SubCommand::with_name(SUB_CMD_GEN_CONF)
                .about("Generate network configuration for specified state")
                .arg(
                    clap::Arg::with_name("STATE_FILE")
                        .required(true)
                        .index(1)
                        .help("Network state file"),
                ),
        )
        .get_matches();
    if let Some(matches) = matches.subcommand_matches(SUB_CMD_GEN_CONF) {
        if let Some(file_path) = matches.value_of("STATE_FILE") {
            print_result_and_exit(gen_conf(&file_path));
        }
    }
}

// Use T instead of String where T has Serialize
fn print_result_and_exit(result: Result<String, CliError>) {
    match result {
        Ok(s) => {
            println!("{}", s);
            std::process::exit(0);
 n       }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn gen_conf(file_path: &str) -> Result<String, CliError> {
    let fd = match std::fs::File::open(file_path) {
        Ok(fd) => fd,
        Err(e) => {
            return Err(CliError {
                msg: format!("Filed to open file {}: {}", file_path, e),
            })
        }
    };
    let net_state: NetState = match serde_yaml::from_reader(fd) {
        Ok(s) => s,
        Err(e) => {
            return Err(CliError {
                msg: format!("Invalid YAML file {}: {}", file_path, e),
            })
        }
    };
    let confs = match net_state.gen_conf() {
        Ok(c) => c,
        Err(e) => {
            return Err(CliError {
                msg: format!("Failled to generate configurations: {}", e),
            });
        }
    };
    match serde_yaml::to_string(&confs) {
        Ok(s) => Ok(s),
        Err(e) => Err(CliError {
            msg: format!("Failed to generate configurations: {}", e),
        }),
    }
}
