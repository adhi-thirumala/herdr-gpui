//! A custom transport cannot be passed to upstream's `machine add`. Save only
//! its standard catalog entry; the normal bridge starts the session on attach.
use super::*;

pub(super) fn save(request: &Request, claim: &Claim) -> Result<()> {
    let config = Config::load()?;
    save_custom(
        request,
        claim,
        |host| Ok(herdr_client::store_saved_host(false, host)?),
        load_catalog,
        Config::save_device_ssh_command,
        |host| resolve_saved(&config, host),
    )
}

/// Keep the command until the catalog's publication outcome is known.
fn save_custom(
    request: &Request,
    claim: &Claim,
    store: impl Fn(SavedHost) -> Result<()>,
    load: impl Fn() -> Result<Vec<SavedHost>>,
    save_command: impl Fn(&str, &SshCommand) -> Result<()>,
    resolve: impl Fn(&SavedHost) -> Option<Destination> + Copy,
) -> Result<()> {
    ensure_unsaved(request, claim, &load()?, resolve)?;
    let id = uuid::Uuid::new_v4().simple().to_string();
    // Publish the executable preference before the catalog can expose this host.
    save_command(&id, &request.target.command)?;
    if let Err(error) = store(SavedHost {
        id: id.clone(),
        label: request.label.clone(),
        target: request.target.target.clone(),
        session: request.session.clone(),
        enabled: true,
    }) {
        // `store_saved_host` atomically renames the catalog into place before
        // syncing its directory, so an append can report an error even though
        // the entry is already visible. Remove the preference only when the ID
        // is provably absent, and keep it when the catalog cannot be read.
        match load() {
            Ok(hosts) if !hosts.iter().any(|host| host.id == id) => {
                if let Err(cleanup) = save_command(&id, &SshCommand::default()) {
                    tracing::warn!(
                        category = "device_setup",
                        %cleanup,
                        "Could not remove the command left by a failed add"
                    );
                }
            }
            Err(source) => tracing::warn!(
                category = "device_setup",
                %source,
                "Could not check the catalog after a failed add; keeping the command"
            ),
            _ => {}
        }
        return Err(error);
    }
    // Upstream's CLI does not lock its catalog. Apply the same oldest-profile
    // rule as ordinary setup if another client saved an alias in between.
    let hosts = load()?;
    if let Some(first) = first_saved(request, claim, &hosts, resolve)
        && first.id != id
    {
        remove(&id)?;
        save_command(&id, &SshCommand::default())?;
        return Err(Error::DeviceExists(first.label.clone()));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
