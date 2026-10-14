use super::*;

#[gpui::test]
fn editing_a_disabled_host_does_not_enable_or_connect_it(cx: &mut TestAppContext) {
    let (view, cx) = window(cx);
    view.update_in(cx, |view, window, cx| {
        let id = "ssh:0123456789abcdef0123456789abcdef";
        view.endpoints[0].connection.target = herdr_client::ConnectTarget::Local;
        view.endpoints.push(crate::endpoint::Endpoint::new(
            id.into(),
            "Devbox".into(),
            herdr_client::ConnectTarget::Ssh {
                target: "me@devbox".into(),
                session: "default".into(),
            },
            false,
        ));
        assert!(matches!(
            ssh_command(view, id, window, cx),
            Response::Done { .. }
        ));
        assert_eq!(view.menu.page, Some(Page::SshCommand));
        assert!(!view.endpoints[1].enabled);
        assert!(view.endpoints[1].connection.handle.is_none());
    });
}
