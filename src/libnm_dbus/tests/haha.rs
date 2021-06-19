use nm_dbus::{ErrorKind, NmApi};

#[test]
fn test_full() {
    let nm = NmApi::new().unwrap();
    println!("NM version: {}", nm.version().unwrap());
    let cp = nm.checkpoint_create().unwrap();
    println!("checkpoint_create: {}", cp);
    let rt = nm.checkpoint_create();
    assert!(rt.is_err());
    if let Err(e) = rt {
        assert!(e.kind == ErrorKind::CheckpointConflict);
    }

    println!("second checkpoint_create() got conflict error as expected");

    nm.checkpoint_destroy(&cp).unwrap();
    println!("checkpoint_destroy: done");
}
