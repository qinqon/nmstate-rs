mod io_nmstate_plugin;
mod nm;
mod plugin;

use crate::plugin::NmPlugin;
use io_nmstate_plugin::{Call_GenerateConfigs, NetState, VarlinkInterface};
use nmstate;
use varlink;

struct NmPluginVarlinkService;

impl VarlinkInterface for NmPluginVarlinkService {
    fn generate_configs(
        &self,
        call: &mut dyn Call_GenerateConfigs,
        net_state: NetState,
    ) -> varlink::Result<()> {
        let logs = vec!["HAHA".to_string()];

        let plugin = match NmPlugin::new() {
            Ok(p) => p,
            Err(e) => return call.reply_nm_state_plugin_err(
                nmstate::ErrorKind::Bug.to_string(),
                e.to_string(),
                logs,
            ),
        };
        match plugin.generate_configs(&net_state) {
            Ok(confs) => call.reply(logs, confs),
            Err(e) => call.reply_nm_state_plugin_err(
                e.kind.to_string(),
                e.msg.to_string(),
                logs,
            ),
        }
    }
}

fn run_server(socket_path: &str) {
    let my_varlink_service = NmPluginVarlinkService;
    let my_varlink_iface = io_nmstate_plugin::new(Box::new(my_varlink_service));
    let service = varlink::VarlinkService::new(
        "io.nmstate.plugin",
        "Nmstate NetworkManager Plugin",
        "0.1",
        "https://nmstate.io",
        vec![Box::new(my_varlink_iface)],
    );

    varlink::listen(
        service,
        &format!("unix:{}", socket_path),
        &varlink::ListenConfig {
            ..Default::default()
        },
    )
    .unwrap();
}

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() != 2 {
        eprintln!("Invalid argument, please specify $1 as socket file path");
        std::process::exit(1);
    }
    run_server(&argv[1]);
}
