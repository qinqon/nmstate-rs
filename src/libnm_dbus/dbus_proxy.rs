use zbus::dbus_proxy;

#[dbus_proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NetworkManager {
    #[dbus_proxy(property)]
    fn version(&self) -> zbus::Result<String>;

    #[dbus_proxy(property)]
    fn active_connections(
        &self,
    ) -> zbus::Result<Vec<zvariant::OwnedObjectPath>>;

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

    /// ActivateConnection method
    fn activate_connection(
        &self,
        connection: &zvariant::ObjectPath,
        device: &zvariant::ObjectPath,
        specific_object: &zvariant::ObjectPath,
    ) -> zbus::Result<zvariant::OwnedObjectPath>;

    /// DeactivateConnection method
    fn deactivate_connection(
        &self,
        active_connection: &zvariant::ObjectPath,
    ) -> zbus::Result<()>;
}

#[dbus_proxy(
    interface = "org.freedesktop.NetworkManager.Settings",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager/Settings"
)]
trait NetworkManagerSetting {
    /// GetConnectionByUuid method
    fn get_connection_by_uuid(
        &self,
        uuid: &str,
    ) -> zbus::Result<zvariant::OwnedObjectPath>;
}
