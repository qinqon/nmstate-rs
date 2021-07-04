mod error;

use clap;
use nmstate::{Interfaces, NetworkState};
use serde::Serialize;
use serde_yaml::{self, Value};

use crate::error::CliError;

const SUB_CMD_GEN_CONF: &str = "gc";
const SUB_CMD_SHOW: &str = "show";

fn main() {
    let matches = clap::App::new("nmstatectl")
        .version("1.0")
        .author("Gris Ge <fge@redhat.com>")
        .about("Command line of nmstate")
        .setting(clap::AppSettings::SubcommandRequired)
        .subcommand(
            clap::SubCommand::with_name(SUB_CMD_SHOW)
                .about("Show network state"),
        )
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
    } else if let Some(_) = matches.subcommand_matches(SUB_CMD_SHOW) {
        print_result_and_exit(show());
    }
}

// Use T instead of String where T has Serialize
fn print_result_and_exit(result: Result<String, CliError>) {
    match result {
        Ok(s) => {
            println!("{}", s);
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn gen_conf(file_path: &str) -> Result<String, CliError> {
    let fd = std::fs::File::open(file_path)?;
    let net_state: NetworkState = serde_yaml::from_reader(fd)?;
    let confs = net_state.gen_conf()?;
    Ok(serde_yaml::to_string(&confs)?)
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct SortedNetworkState {
    interfaces: Vec<Value>,
}

const IFACE_TOP_PRIORTIES: [&str; 2] = ["name", "type"];

fn sort_netstate(
    net_state: NetworkState,
) -> Result<SortedNetworkState, CliError> {
    let mut ifaces = net_state.interfaces.unwrap_or(Interfaces::new()).to_vec();
    ifaces.sort_by(|a, b| a.name().cmp(b.name()));

    if let Value::Sequence(ifaces) = serde_yaml::to_value(&ifaces)? {
        let mut new_ifaces = Vec::new();
        for iface_v in ifaces {
            if let Value::Mapping(iface) = iface_v {
                let mut new_iface = serde_yaml::Mapping::new();
                for top_property in IFACE_TOP_PRIORTIES {
                    if let Some(v) =
                        iface.get(&Value::String(top_property.to_string()))
                    {
                        new_iface.insert(
                            Value::String(top_property.to_string()),
                            v.clone(),
                        );
                    }
                }
                for (k, v) in iface.iter() {
                    if let Value::String(ref name) = k {
                        if IFACE_TOP_PRIORTIES.contains(&name.as_str()) {
                            continue;
                        }
                    }
                    new_iface.insert(k.clone(), v.clone());
                }

                new_ifaces.push(Value::Mapping(new_iface));
            }
        }
        return Ok(SortedNetworkState {
            interfaces: new_ifaces,
        });
    }

    Ok(SortedNetworkState {
        interfaces: Vec::new(),
    })
}

// Ordering the outputs
fn show() -> Result<String, CliError> {
    let sorted_net_state = sort_netstate(NetworkState::retrieve()?)?;
    Ok(serde_yaml::to_string(&sorted_net_state)?)
}
