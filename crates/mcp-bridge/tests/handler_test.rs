use metaygn_mcp_bridge::DaemonClient;

#[test]
fn daemon_client_constructs() {
    let _client = DaemonClient::new(12345).unwrap();
    // Just verify it creates without panic
    assert!(true);
}
