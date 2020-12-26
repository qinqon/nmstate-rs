mod io_nmstate_plugin;

use io_nmstate_plugin::{Call_GenerateConfigs, NetState, VarlinkInterface};
use varlink;

struct NmPlugin;

impl VarlinkInterface for NmPlugin {
    fn generate_configs(
        &self,
        call: &mut dyn Call_GenerateConfigs,
        net_state: NetState,
    ) -> varlink::Result<()> {
        let logs = vec!["HAHA".to_string()];
        let confs = vec!["CONF1".to_string()];
        return call.reply(logs, confs);
    }
}

fn run_server(socket_path: &str) {
    let nm_plugin = NmPlugin;
    let my_varlink_iface = io_nmstate_plugin::new(Box::new(nm_plugin));
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
