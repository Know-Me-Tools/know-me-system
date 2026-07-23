//! Contributor-world fixture guest (task 7.2).
//!
//! Returns one provenance-tagged `system-prompt-fragment` contribution, with
//! the requested stage echoed into the payload so the host test can verify
//! the input round-tripped.

wit_bindgen::generate!({
    path: "../../../wit/v1",
    world: "contributor",
});

use knowme::plugin::contribution_types::{ContributionItem, CustomItem};

struct ContributorGuest;

impl Guest for ContributorGuest {
    fn contribute(input: ContributionInput) -> Result<Contribution, String> {
        let stage = format!("{:?}", input.stage);
        Ok(Contribution {
            items: vec![ContributionItem::Custom(CustomItem {
                source: "knowme.fixture.contributor".to_string(),
                name: "system-prompt-fragment".to_string(),
                payload_json: format!(
                    "{{\"stage\":\"{stage}\",\"fragment\":\"You are a concise assistant.\"}}"
                ),
            })],
        })
    }
}

impl exports::knowme::plugin::lifecycle::Guest for ContributorGuest {
    fn init() -> Result<(), String> {
        Ok(())
    }

    fn shutdown() {}
}

export!(ContributorGuest);
