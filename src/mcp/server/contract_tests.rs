use super::*;

#[test]
fn advertised_spec_contracts_distinguish_store_selector_from_reference_base() {
    let tools = SpecServer::tool_router().list_all();
    assert_eq!(tools.len(), 19);
    for tool in &tools {
        let value = serde_json::to_value(tool).unwrap();
        assert!(
            value["inputSchema"]["properties"]["workspace"]["description"]
                .as_str()
                .is_some()
        );
    }
    let update = tools
        .iter()
        .find(|tool| tool.name == "spec_update")
        .unwrap();
    let value = serde_json::to_value(update).unwrap();
    assert!(
        value["inputSchema"]["properties"]["workspace"]["description"]
            .as_str()
            .unwrap()
            .contains("Required explicit")
    );
    let refs = tools
        .iter()
        .find(|tool| tool.name == "spec_refs_validate")
        .unwrap();
    assert!(
        refs.description
            .as_deref()
            .unwrap()
            .contains("not the workspace store selector")
    );
}
