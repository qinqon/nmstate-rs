## Plugin design
 * All plugins are executable.
 * The search folder of nmstate system variable `NMSTATE_PLUGIN_FOLDER` and
   default to `/usr/bin`.
 * Plugin is named `nmstate_plugin_<plugin_name>`.
 * Plugin will take first argument as varlink socket file path.
 * `libnmstate` will retry 50 times with 0.1 seconds interval to wait
   plugin varlink interface up.
 * The `libnmstate` will invoke the plugin as child thread and communicate
   with it varlink and terminate this child thread once done.
 * The varlink interface file is `src/libnmstate/io.nmstate.plugin.varlink`.

## Use plugin build from git repo

```bash
cargo build
sudo env NMSTATE_PLUGIN_FOLDER=./target/debug/ \
    ./target/debug/nmstatectl gc ~/ymls/eth/eth1_up.yml
```
