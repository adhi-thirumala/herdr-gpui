use super::*;
use crate::config::{Config, DeviceSettings};
use crate::orchestrator::actions::saved_command;
use herdr_client::{SavedHost, SshCommand};

#[test]
fn saved_run_commands_follow_device_identity_and_preserve_session_selection() -> anyhow::Result<()>
{
    let mut config = Config::default();
    let first = SavedHost {
        id: "a".repeat(32),
        label: "Build".into(),
        target: "build".into(),
        session: "default".into(),
        enabled: true,
    };
    let mut second = first.clone();
    second.id = "b".repeat(32);
    second.session = "other".into();
    for (host, command) in [(&first, "tailscale ssh"), (&second, "ssh -F other.conf")] {
        config.devices.insert(
            host.id.clone(),
            DeviceSettings {
                ssh_command: SshCommand::try_from(command)?,
                ..DeviceSettings::default()
            },
        );
    }
    let mut on = session("run");
    on.host = Some("build".into());
    on.session = Some("picked-after-saving".into());
    assert_eq!(
        saved_command(&on, std::slice::from_ref(&first), &config)?.as_str(),
        "tailscale ssh"
    );
    let hosts = [first, second];
    assert!(matches!(
        saved_command(&on, &hosts, &config),
        Err(Error::RunHostUnavailable)
    ));
    on.session = Some("other".into());
    assert_eq!(
        saved_command(&on, &hosts, &config)?.as_str(),
        "ssh -F other.conf"
    );
    config
        .devices
        .entry(hosts[1].id.clone())
        .or_default()
        .ssh_command = config.device_ssh_command(&hosts[0].id);
    on.session = Some("picked-after-saving".into());
    assert_eq!(
        saved_command(&on, &hosts, &config)?.as_str(),
        "tailscale ssh"
    );
    on.host = Some("removed".into());
    assert!(matches!(
        saved_command(&on, &hosts, &config),
        Err(Error::RunHostUnavailable)
    ));
    Ok(())
}
