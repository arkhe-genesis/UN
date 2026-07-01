pub enum TypedAction {
    NoAction,
    ReadPathAction { path: String },
}

pub struct ContainmentLayer;
impl ContainmentLayer {
    pub fn validate(&self, _action: &TypedAction) -> Result<(), &'static str> { Ok(()) }
}
