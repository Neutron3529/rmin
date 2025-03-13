use crate::{fmt, Display};
/// Show the caption (and label) of a table
#[derive(Default)]
pub struct Caption {
    pub cap: Option<String>,
    pub label: Option<String>,
}
impl Display for Caption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.cap.as_ref(), self.label.as_ref()) {
            (Some(c), Some(l)) => write!(f, r#"\caption{{{c}\label{{{l}}}}}"#),
            (Some(c), None) => write!(f, r#"\caption{{{c}}}"#),
            (None, Some(l)) => write!(f, r#"\caption{{\label{{{l}}}}}"#),
            (None, None) => Ok(()),
        }
    }
}
