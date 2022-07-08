use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TODO: remove pub(crate) and impl IntoIter
#[derive(Debug, Deserialize)]
pub struct Lights(pub(crate) HashMap<u32, Light>);

impl Lights {
    pub fn name(&mut self, name: &str) -> &mut Light {
        self.0
            .iter_mut()
            .find(|(_, light)| light.name == name)
            .unwrap()
            .1
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct LightChange {
    #[serde(skip_serializing_if = "Option::is_none")]
    on: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Light {
    name: String,
    state: State,
    pub(crate) change: Option<LightChange>,
}

impl Light {
    pub fn toggle(&mut self) {
        self.state.on = !self.state.on;
        self.change.get_or_insert(LightChange::default()).on = Some(self.state.on);
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct State {
    on: bool,
}
