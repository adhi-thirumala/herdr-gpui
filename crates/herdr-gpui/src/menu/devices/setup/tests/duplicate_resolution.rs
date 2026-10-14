use super::*;

#[test]
fn duplicate_checks_use_each_saved_devices_configured_destination() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let executable = dir.path().join("ssh");
    crate::test_executable::write(
        &executable,
        r#"#!/bin/sh
for arg do
    case "$arg" in
        prod.conf) host=machine-a ;;
        build.conf) host=machine-b ;;
    esac
done
printf 'hostname %s\nuser penso\nport 22\n' "$host"
"#,
        0o700,
    )?;
    let command = |file| {
        SshCommand::try_from(format!(
            "{} -F {file}",
            herdr_client::shell_quote(&executable.to_string_lossy())
        ))
    };
    let request = Request::new("build", "Build", "", command("build.conf")?.as_str())?;
    let claim = Claim::acquire(destination("machine-b"), "default")?;
    let hosts = [saved('a', "Production", "prod")];
    let mut config = Config::default();
    config.devices.insert(
        hosts[0].id.clone(),
        crate::config::DeviceSettings {
            ssh_command: command("prod.conf")?,
            ..crate::config::DeviceSettings::default()
        },
    );
    ensure_unsaved(&request, &claim, &hosts, |host| {
        resolve_saved(&config, host)
    })?;
    config
        .devices
        .entry(hosts[0].id.clone())
        .or_default()
        .ssh_command = command("build.conf")?;
    assert!(matches!(
        ensure_unsaved(&request, &claim, &hosts, |host| resolve_saved(&config, host)),
        Err(Error::DeviceExists(label)) if label == "Production"
    ));
    Ok(())
}
