mod error;

use clap;
use env_logger::Builder;
use log::LevelFilter;
use nmstate::NetworkState;
use serde::Serialize;
use serde_yaml::{self, Value};

use crate::error::CliError;

const SUB_CMD_GEN_CONF: &str = "gc";
const SUB_CMD_SHOW: &str = "show";
const SUB_CMD_APPLY: &str = "apply";

fn main() {
    let matches = clap::App::new("nmstatectl")
        .version("1.0")
        .author("Gris Ge <fge@redhat.com>")
        .about("Command line of nmstate")
        .setting(clap::AppSettings::SubcommandRequired)
        .arg(
            clap::Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .help("Set verbose level"),
        )
        .subcommand(
            clap::SubCommand::with_name(SUB_CMD_SHOW)
                .about("Show network state")
                .arg(
                    clap::Arg::with_name("IFNAME")
                        .index(1)
                        .help("Show speific interface only"),
                )
                .arg(
                    clap::Arg::with_name("KERNEL")
                        .short("k")
                        .long("kernel")
                        .takes_value(false)
                        .help("Show kernel network state only"),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name(SUB_CMD_APPLY)
                .about("Apply network state")
                .arg(
                    clap::Arg::with_name("STATE_FILE")
                        .required(true)
                        .index(1)
                        .help("Network state file"),
                )
                .arg(
                    clap::Arg::with_name("KERNEL")
                        .short("k")
                        .long("kernel")
                        .takes_value(false)
                        .help("Apply network state to kernel only"),
                ),
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
    let (log_module_filter, log_level) = match matches.occurrences_of("verbose")
    {
        0 => (Some("nmstate"), LevelFilter::Warn),
        1 => (Some("nmstate"), LevelFilter::Info),
        2 => (Some("nmstate"), LevelFilter::Debug),
        _ => (None, LevelFilter::Debug),
    };

    let mut log_builder = Builder::new();
    log_builder.filter(log_module_filter, log_level);
    log_builder.init();

    if let Some(matches) = matches.subcommand_matches(SUB_CMD_GEN_CONF) {
        if let Some(file_path) = matches.value_of("STATE_FILE") {
            print_result_and_exit(gen_conf(&file_path));
        }
    } else if let Some(matches) = matches.subcommand_matches(SUB_CMD_SHOW) {
        print_result_and_exit(show(&matches));
    } else if let Some(matches) = matches.subcommand_matches(SUB_CMD_APPLY) {
        if let Some(file_path) = matches.value_of("STATE_FILE") {
            print_result_and_exit(apply(
                &file_path,
                matches.is_present("KERNEL"),
            ));
        }
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
    let mut ifaces = net_state.interfaces.to_vec();
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
fn show(matches: &clap::ArgMatches) -> Result<String, CliError> {
    let mut net_state = NetworkState::new();
    if matches.is_present("KERNEL") {
        net_state.set_kernel_only(true);
    }
    net_state.retrieve()?;
    Ok(if let Some(ifname) = matches.value_of("IFNAME") {
        let mut new_net_state = NetworkState::new();
        new_net_state.set_kernel_only(matches.is_present("KERNEL"));
        for iface in net_state.interfaces.to_vec() {
            if iface.name() == ifname {
                new_net_state.append_interface_data(iface.clone())
            }
        }
        serde_yaml::to_string(&new_net_state)?
    } else {
        serde_yaml::to_string(&sort_netstate(net_state)?)?
    })
}

fn apply(file_path: &str, kernel_only: bool) -> Result<String, CliError> {
    let fd = std::fs::File::open(file_path)?;
    let mut net_state: NetworkState = serde_yaml::from_reader(fd)?;
    net_state.set_kernel_only(kernel_only);
    net_state.apply()?;
    let sorted_net_state = sort_netstate(net_state)?;
    Ok(serde_yaml::to_string(&sorted_net_state)?)
}
