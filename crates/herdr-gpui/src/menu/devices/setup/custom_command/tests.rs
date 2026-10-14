use super::*;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

#[test]
fn failed_append_removes_only_a_provably_unpublished_command() -> Result<()> {
    for published in [Some(false), Some(true), None] {
        let request = Request::new("build", "Build", "", "tailscale ssh")?;
        let claim = Claim::fixture("custom-command-rollback");
        let attempted = Cell::new(false);
        let hosts = RefCell::new(Vec::new());
        let commands = RefCell::new(BTreeMap::new());
        let expected = RefCell::new(BTreeMap::new());
        let result = save_custom(
            &request,
            &claim,
            |host| {
                attempted.set(true);
                if published != Some(false) {
                    expected
                        .borrow_mut()
                        .insert(host.id.clone(), request.target.command.clone());
                }
                if published == Some(true) {
                    hosts.borrow_mut().push(host);
                }
                Err(Error::DeviceSetupInput("append failed"))
            },
            || {
                if attempted.get() && published.is_none() {
                    Err(Error::DeviceSetupInput("catalog unreadable"))
                } else {
                    Ok(hosts.borrow().clone())
                }
            },
            |id, command| {
                if command.is_default() {
                    commands.borrow_mut().remove(id);
                } else {
                    commands.borrow_mut().insert(id.to_owned(), command.clone());
                }
                Ok(())
            },
            |_| None,
        );
        assert!(matches!(
            result,
            Err(Error::DeviceSetupInput("append failed"))
        ));
        assert_eq!(*commands.borrow(), *expected.borrow());
    }
    Ok(())
}
