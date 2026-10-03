#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FormAction {
    Submit,
    Reset,
}

impl FormAction {
    pub fn key(&self) -> &'static str {
        match self {
            Self::Submit => "form_action.submit",
            Self::Reset => "form_action.reset",
        }
    }
}
