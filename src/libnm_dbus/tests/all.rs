use nm_dbus::{ErrorKind, NmApi};

const TEST_NM_CONNECTION_CONTENT: &str = r#"
[connection]
id=br0
uuid=7300353a-0a72-4815-a12c-4720b6c2f1a0
type=bridge
interface-name=br0

[bridge]

[ipv4]
method=disabled

[ipv6]
method=disabled
"#;

#[test]
fn test_full() {
    let nm = NmApi::new().unwrap();
    println!("NM version: {}", nm.version().unwrap());
    let cp = nm.checkpoint_create().unwrap();
    println!("checkpoint_create: {}", cp);
    let e = nm.checkpoint_create().unwrap_err();
    assert!(e.kind == ErrorKind::CheckpointConflict);

    println!("second checkpoint_create() got conflict error as expected");

    nm.checkpoint_destroy(&cp).unwrap();
    println!("checkpoint_destroy: done");

    let cp = nm.checkpoint_create().unwrap();
    println!("checkpoint_create: {}", cp);

    nm.add_connection("br0", TEST_NM_CONNECTION_CONTENT)
        .unwrap();
    let conn_uuid = "7300353a-0a72-4815-a12c-4720b6c2f1a0";
    nm.activate(conn_uuid).unwrap();
    println!("connection {} activated", conn_uuid);

    std::thread::sleep(std::time::Duration::from_millis(100));
    nm.deactivate(conn_uuid).unwrap();
    println!("connection {} deactivated", conn_uuid);

    nm.checkpoint_rollback(&cp).unwrap();
    println!("checkpoint_rollback: done");
}
