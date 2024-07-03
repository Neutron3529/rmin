/// Printable data with flags.
use crate::{fmt, Display};
#[derive(Default)]
pub struct Data<T: Display + Default> {
    pub data: T,
    pub rounding: u32,
    pub as_percentage: bool,
    pub as_bold: bool,
    pub as_italic: bool,
    pub stars: u32,
}
impl Data<f64> {
    fn from<'a>(data: impl IntoIterator<Item = &'a f64>) -> Vec<Self> {
        Vec::from_iter(data.into_iter().map(|&x| Data {
            data: x,
            ..Default::default()
        }))
    }
}

impl Display for Data<f64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{ipre}{bpre}{item:0.rounding$}{bpo}{ipo}{percentage}{placeholder:*>stars$}",
            placeholder = "",
            item = self.data,
            rounding = self.rounding as usize,
            percentage = if self.as_percentage { "%" } else { "" },
            ipre = if self.as_italic { r#"\it{"# } else { "" },
            bpre = if self.as_bold { r#"\bold{"# } else { "" },
            ipo = if self.as_italic { r#"}"# } else { "" },
            bpo = if self.as_bold { r#"}"# } else { "" },
            stars = self.stars as usize
        )
    }
}
