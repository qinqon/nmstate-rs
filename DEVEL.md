# Use plugin build from git repo

```bash
cargo build
sudo env NMSTATE_PLUGIN_FOLDER=./target/debug/ \
    ./target/debug/nmstatectl gc ~/ymls/eth/eth1_up.yml
```
