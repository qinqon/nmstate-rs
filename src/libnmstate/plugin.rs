use crate::error::NmstateError;
use crate::io_nmstate_plugin::VarlinkClient;
use crate::io_nmstate_plugin::VarlinkClientInterface;
use crate::NetState;
use std::os::unix::fs::PermissionsExt;

const DEFAULT_PLUGIN_SEARCH_FOLDER: &str = "/usr/bin";
const PLUGIN_PREFIX: &str = "nmstate_plugin_";
const VARLINK_SOCKET_PREFIX: &str = "/run/nmstate_plugin_";

const MAX_VARLINK_CONNECTION_RETRY: u8 = 50;
const VALINK_CONNECTION_RETRY_INTERVAL: u64 = 100; // miliseconds

pub(crate) struct NmstatePlugin {
    pub(crate) name: String,
    child: std::process::Child,
    varlink_conn: VarlinkClient,
}

impl NmstatePlugin {
    fn start(
        plugin_exec_path: &str,
        plugin_name: &str,
    ) -> Result<Self, NmstateError> {
        let socket_path = format!("{}{}", VARLINK_SOCKET_PREFIX, plugin_name);
        // Invoke the plugin in child.
        match std::process::Command::new(plugin_exec_path)
            .arg(&socket_path)
            .spawn()
        {
            Ok(mut child) => {
                println!("DEBUG: Plugin {} started", plugin_exec_path);
                let varlink_conn = match connect_varlink(&socket_path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!(
                            "Failed to connect the varlink interface of \
                             plugin {}: {}",
                            plugin_name, &e
                        );
                        stop_child(&mut child, plugin_name);
                        return Err(NmstateError::plugin_failure(format!(
                            "Failed to connect the varlink interface of \
                             plugin {}: {}",
                            plugin_name, &e
                        )));
                    }
                };
                Ok(NmstatePlugin {
                    name: plugin_name.into(),
                    child: child,
                    varlink_conn: varlink_conn,
                })
            }
            Err(e) => Err(NmstateError::invalid_argument(format!(
                "Failed to start plugin {} {}: {}",
                plugin_exec_path, &socket_path, e
            ))),
        }
    }

    pub(crate) fn stop(&mut self) {
        stop_child(&mut self.child, &self.name);
    }

    pub(crate) fn load_plugins() -> Result<Vec<Self>, NmstateError> {
        let mut plugins = Vec::new();
        let search_folder = match std::env::var("NMSTATE_PLUGIN_FOLDER") {
            Ok(d) => d,
            Err(_) => DEFAULT_PLUGIN_SEARCH_FOLDER.to_string(),
        };
        match std::fs::read_dir(&search_folder) {
            Ok(dir) => {
                for entry in dir {
                    let file_name = match entry {
                        Ok(f) => f.file_name(),
                        Err(e) => {
                            eprintln!("FAIL: Failed to read dir entry: {}", e);
                            continue;
                        }
                    };
                    let file_name = match file_name.to_str() {
                        Some(n) => n,
                        None => {
                            eprintln!("BUG: Failed to read file_name",);
                            continue;
                        }
                    };
                    if file_name.starts_with(PLUGIN_PREFIX) {
                        let plugin_exec_path =
                            format!("{}/{}", &search_folder, file_name);
                        if !is_executable(&plugin_exec_path) {
                            continue;
                        }
                        let plugin_name =
                            match file_name.strip_prefix(PLUGIN_PREFIX) {
                                Some(n) => n,
                                None => {
                                    eprintln!(
                                        "BUG: file_name {} not started with {}",
                                        file_name, PLUGIN_PREFIX,
                                    );
                                    continue;
                                }
                            };
                        println!("DEBUG: Found plugin {}", &plugin_exec_path);
                        match NmstatePlugin::start(
                            &plugin_exec_path,
                            &plugin_name,
                        ) {
                            Ok(plugin) => plugins.push(plugin),
                            Err(e) => {
                                eprintln!("{}", e);
                                continue;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                return Err(NmstateError::invalid_argument(format!(
                    "Faild to open plugin search dir /usr/bin: {}",
                    e
                )));
            }
        }
        Ok(plugins)
    }

    pub(crate) fn gen_conf(
        &mut self,
        net_state: &NetState,
    ) -> Result<Vec<String>, NmstateError> {
        match self.varlink_conn.generate_configs(net_state.clone()).call() {
            Ok(reply) => {
                for log in reply.logs {
                    println!("DEBUG: plugin {}: {}", &self.name, log);
                }
                Ok(reply.confs)
            }
            Err(e) => Err(NmstateError::plugin_failure(format!(
                "Failed to invoke generate_configs() on plugin {}: {}",
                self.name, e
            ))),
        }
    }
}

fn is_executable(file_path: &str) -> bool {
    if let Ok(attr) = std::fs::metadata(file_path) {
        attr.permissions().mode() & 0o100 != 0
    } else {
        false
    }
}

fn connect_varlink(socket_path: &str) -> Result<VarlinkClient, NmstateError> {
    let mut i = 0u8;
    while i < MAX_VARLINK_CONNECTION_RETRY {
        let connection = match varlink::Connection::with_address(&format!(
            "unix:{}",
            socket_path
        )) {
            Ok(c) => c,
            Err(e) => {
                println!(
                    "DEBUG: varlink connection to plugin {} retry {}/{}",
                    socket_path,
                    i + 1,
                    MAX_VARLINK_CONNECTION_RETRY
                );
                if i == MAX_VARLINK_CONNECTION_RETRY - 1 {
                    return Err(NmstateError::invalid_argument(format!(
                        "Failed to connect plugin varlink interface {}: {}",
                        socket_path, e
                    )));
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(
                        VALINK_CONNECTION_RETRY_INTERVAL,
                    ));
                    i = i + 1;
                    continue;
                }
            }
        };
        return Ok(VarlinkClient::new(connection));
    }
    Err(NmstateError::invalid_argument(format!(
        "Failed to connect plugin varlink interface {}: timeout",
        socket_path,
    )))
}

fn stop_child(child: &mut std::process::Child, name: &str) {
    let id = child.id();
    if let Err(e) = child.kill() {
        eprintln!("Failed to kill plugin {} child process {}: {}", name, id, e);
    }
    child.wait();
    println!("DEBUG: plugin {} child process {} terminated", name, id);
}
