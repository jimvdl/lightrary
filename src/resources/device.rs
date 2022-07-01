use serde::ser::{Serialize, Serializer, SerializeStruct};

#[derive(Debug)]
pub struct DeviceType<'a> {
    pub app_name: &'a str,
    pub instance_name: &'a str,
}

impl<'a> Serialize for DeviceType<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("DeviceType", 2)?;
        state.serialize_field("devicetype", &format!("{}#{}", self.app_name, self.instance_name))?;
        state.serialize_field("generateclientkey", &true)?;
        state.end()
    }
}
