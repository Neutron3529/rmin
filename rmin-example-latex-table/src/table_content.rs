use crate::{fmt, Data, Display, Rows};
/// Table content
///     \begin{tabular} % defined in Table
///         \toprules % defined in Table
///             % col_name[0] .. col_name[last]    template_rows[0] .. template_rows[last]
///             meta                            &
///         \bottomrules % defined in Table
///     \end{tabular} % defined in Table
#[derive(Default)]
pub struct TableContent {
    /// real data
    pub data: Vec<f64>,
    // /// real data: data cols
    // pub data_cols:usize,
    /// col template
    pub data_rows: Rows,
    /// col names, `self.cols()` equals to `self.col_name.len()`.
    pub col_name: Vec<String>,
    /// store meta informations by row-first storage.
    pub meta: Option<String>,
    /// row names
    pub template_rows: Rows,
    /// extra hline
    pub hline: Vec<usize>,
    /// extra cline
    pub cline: Vec<(usize, usize, usize)>,
}
impl TableContent {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn format(&self) -> String {
        format!("{}{}", self.template_rows, self.data_rows)
    }
    pub fn set_data<T: Into<Vec<f64>> + Sized, U: Into<Vec<String>> + Sized>(
        &mut self,
        data: T,
        col_name: U,
    ) {
        self.data = data.into();
        self.col_name = col_name.into();
    }
    pub fn cols(&self) -> usize {
        self.col_name.len()
    }
}
impl Display for TableContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.template_rows.names.iter().chain(self.data_rows.names.iter());
        if let Some(item) = iter.next_back() {
            write!(f, "\n        ")?;
            for item in iter {
                write!(f,"{item} & ")?
            }
            write!(f,r"{item}\\")?
        }
        let col_len = self.col_name.len();
        let row_len = self.data_rows.names.len();
        if col_len * row_len != self.data.len() {
            write!(f,"%<Error: template accept {col_len} x {row_len} matrix, but only {} provided>", self.data.len())?;
        } else {
            let mut data = Data::from(&self.data);
            
            for j in 0..row_len {
                let r = &self.data_rows.rows[j];
                for i in 0..col_len {
                    data[j*col_len + i].rounding = r.ifn.rounding;
                    data[j*col_len + i].as_percentage = r.ifn.as_percentage;
                }
            }
            
            
            for i in 0..col_len {
                write!(f, "\n        {} & ", self.col_name[i])?;
                for j in 0..row_len {
                    write!(f,"{}", data[i+j*col_len])?;
                    if j != row_len - 1 {
                        write!(f, " & ")?
                    } else {
                        write!(f, r"\\")?
                    }
                }
            }
        }
        Ok(())
    }
}
