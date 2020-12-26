 * All plugin are executable.
 * Plugin is named `/usr/bin/nmstate_plugin_<plugin_name>`.
 * Plugin will take first argument as varlink socket file path.
 * The `libnmstate` will invoke the plugin as child thread and communicate
   with it varlink and terminate this child thread once done.
 * The plugin should provides:
    * `io.nmstate.GenerateConfig(net_state)` -> input json of `NetState`,
      output of `HashMap<String, [String]>`
    * `io.nmstate.GetNetState()` -> output JSON of `NetState`.
    * `io.nmstate.Apply(net_state)` -> input json of `NetState`
