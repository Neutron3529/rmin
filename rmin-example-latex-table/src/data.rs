/// Printable data with flags.
use crate::{fmt, Display};
#[derive(Default, Clone, Copy)]
pub struct Data<T: Display + Default> {
    pub data: T,
    pub rounding: i32,
    pub as_percentage: bool,
    pub as_bold: bool,
    pub as_italic: bool,
    pub use_threeparttable: bool,
    pub stars: i32,
}
impl Data<f64> {
    pub fn from<'a>(data: impl IntoIterator<Item = &'a f64>) -> Vec<Self> {
        Vec::from_iter(data.into_iter().map(|&x| Data {
            data: x,
            rounding: -1,
            ..Default::default()
        }))
    }
}

impl Display for Data<f64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{ipre}{bpre}{item:0.rounding$}{bpo}{ipo}{percentage}{stars}",
            item = if self.as_percentage {
                100. * self.data
            } else {
                self.data
            },
            rounding = if self.as_percentage {
                0.max((self.rounding - 2) as isize) as usize
            } else {
                self.rounding as usize
            },
            percentage = if self.as_percentage { "%" } else { "" },
            ipre = if self.as_italic { r#"\textit{"# } else { "" },
            bpre = if self.as_bold { r#"\textbf{"# } else { "" },
            ipo = if self.as_italic { r#"}"# } else { "" },
            bpo = if self.as_bold { r#"}"# } else { "" },
            stars = if self.stars <= 0 {
                String::new()
            } else if self.use_threeparttable {
                format!(
                    "\\tnote{{{stars}*}}",
                    stars = r"{*\!}".repeat(self.stars as usize - 1)
                )
            } else {
                format!(
                    "$^{{{{\\hspace{{-0.1em}}}}^{{{stars}{back}}}}}$",
                    back = r"\!".repeat(2 * self.stars as usize),
                    stars = r"{*\!}".repeat(self.stars as usize),
                )
            }
        )
    }
}
