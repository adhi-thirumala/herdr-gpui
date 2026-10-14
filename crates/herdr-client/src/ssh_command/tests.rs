#![allow(clippy::unwrap_used)]
use super::*;

fn parse(text: &str) -> Result<SshCommand> {
    SshCommand::try_from(text)
}

#[test]
fn invalid_commands_are_typed_errors() {
    for text in [
        "-oBatchMode=yes",
        "'' host",
        "ssh\nhost",
        "ssh\0host",
        "ssh\u{1b}host",
        "# no executable",
        "ssh 'host",
    ] {
        assert!(matches!(parse(text), Err(Error::InvalidSshCommand { .. })));
    }
    // Word count bound.
    let too_many_words = std::iter::repeat_n("x", 65).collect::<Vec<_>>().join(" ");
    assert!(matches!(
        parse(&too_many_words),
        Err(Error::InvalidSshCommand { .. })
    ));
    // Byte bound.
    let too_long = format!("ssh {}", "x".repeat(4096));
    assert!(matches!(
        parse(&too_long),
        Err(Error::InvalidSshCommand { .. })
    ));
}

#[test]
fn settings_deserialization_refuses_invalid_commands() {
    assert!(serde_json::from_str::<SshCommand>("\"-oProxyCommand=x\"").is_err());
    assert!(serde_json::from_str::<SshCommand>("\"ssh 'host\"").is_err());
}
