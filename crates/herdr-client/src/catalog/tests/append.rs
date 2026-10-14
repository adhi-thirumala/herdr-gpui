use super::*;
use serde_json::json;

fn host(id: &str, target: &str, session: &str) -> SavedHost {
    SavedHost {
        id: id.into(),
        label: "Host".into(),
        target: target.into(),
        session: session.into(),
        enabled: true,
    }
}

fn id(n: u64) -> String {
    format!("{n:032x}")
}

fn scratch(name: &str) -> PathBuf {
    let root = env::temp_dir().join(format!(
        "herdr-catalog-append-{name}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    root.join("endpoints.json")
}

#[test]
fn append_preserves_existing_profiles_selection_and_order() {
    let catalog = scratch("preserve");
    let first = host(&id(1), "a@one.invalid", "default");
    let second = host(&id(2), "b@two.invalid", "default");
    fs::write(
        &catalog,
        json!({"version":1,"selected_profile":id(2),"ssh":[first,second]}).to_string(),
    )
    .unwrap();
    let third = host(&id(3), "c@three.invalid", "work");
    store_host(&catalog, third.clone()).unwrap();

    let loaded = load_catalog(&catalog).unwrap();
    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.selected_profile.as_deref(), Some(id(2).as_str()));
    assert_eq!(loaded.ssh, [first, second, third]);

    fs::remove_dir_all(catalog.parent().unwrap()).unwrap();
}

#[test]
fn append_rejects_duplicate_ids_and_target_session_pairs() {
    let catalog = scratch("duplicate");
    let existing = host(&id(1), "a@one.invalid", "default");
    fs::write(&catalog, json!({"version":1,"ssh":[existing]}).to_string()).unwrap();
    // Same ID under another target is a duplicate profile ID.
    assert!(matches!(
        store_host(&catalog, host(&id(1), "b@two.invalid", "default")),
        Err(Error::ProfileId)
    ));
    // A fresh ID for the same host and session is a duplicate endpoint.
    assert!(matches!(
        store_host(&catalog, host(&id(2), "a@one.invalid", "default")),
        Err(Error::DuplicateHost)
    ));
    // A different session for the same target is a distinct endpoint.
    let different_session = host(&id(2), "a@one.invalid", "work");
    store_host(&catalog, different_session.clone()).unwrap();
    assert_eq!(
        load_catalog(&catalog).unwrap().ssh,
        [existing, different_session]
    );
    fs::remove_dir_all(catalog.parent().unwrap()).unwrap();
}

#[test]
fn append_enforces_the_catalog_count_bound() {
    let catalog = scratch("count");
    let hosts: Vec<SavedHost> = (0..64)
        .map(|n| host(&id(n + 1), &format!("h{n}@one.invalid"), "default"))
        .collect();
    fs::write(&catalog, json!({"version":1,"ssh":hosts}).to_string()).unwrap();
    assert!(matches!(
        store_host(&catalog, host(&id(65), "over@one.invalid", "default")),
        Err(Error::CatalogVersionOrCount)
    ));
    fs::remove_dir_all(catalog.parent().unwrap()).unwrap();
}
