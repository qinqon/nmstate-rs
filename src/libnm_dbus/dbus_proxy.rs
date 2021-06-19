use zbus::dbus_proxy;

#[dbus_proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NetworkManager {
    #[dbus_proxy(property)]
    fn version(&self) -> zbus::Result<String>;

    /// CheckpointCreate method
    fn checkpoint_create(
        &self,
        devices: &[zvariant::ObjectPath],
        rollback_timeout: u32,
        flags: u32,
    ) -> zbus::Result<zvariant::OwnedObjectPath>;

    /// CheckpointDestroy method
    fn checkpoint_destroy(
        &self,
        checkpoint: &zvariant::ObjectPath,
    ) -> zbus::Result<()>;
}
