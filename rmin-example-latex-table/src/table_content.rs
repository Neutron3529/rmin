use crate::{fmt, Display, Rows};
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
    pub data:Vec<f64>,
    // /// real data: data cols
    // pub data_cols:usize,
    /// col template
    pub name_rows: Rows,
    /// col names, `self.cols()` equals to `self.col_name.len()`.
    pub col_name:Vec<String>,
    /// store meta informations by row-first storage.
    pub meta : Option<String>,
    /// row names
    pub template_rows: Rows,
    /// extra hline
    pub hline:Vec<usize>,
    /// extra cline
    pub cline:Vec<(usize,usize,usize)>,
}
impl TableContent {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn format(&self) -> String {
        format!("{}{}", self.template_rows, self.name_rows)
    }
    pub fn set_data<T:Into<Vec<f64>>+Sized, U:Into<Vec<String>>+Sized>(&mut self, data:T, col_name:U){
        self.data=data.into();
        self.col_name = col_name.into();
    }
    pub fn cols(&self)->usize{
        self.col_name.len()
    }
}
impl Display for TableContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in self.header {
            write!(f,"{i}")?
        }
        let data_rows = self.data.len()/self.data_cols;
        if data_rows != self.format.row.len() {
            write!(f,"<Error: data row count does not matches the template row count.>")?;
        } else if self.col_name.len() != self.data_cols {
            write!(f,"<Error: data column count does not matches the template column count.>")?;
        } else {
            for i in 0..self.data_cols {
                write!(f, "        {} & ",col_name[i])?;
                for j in 0..data_rows {
                    let data = self.data[i+j*self.data_cols];
                    self.format.row[j].format
                    if j != data_rows - 1 {
                        write!(f, " & ")?
                    } else {
                        write!(f, "\\\\\n")?
                    }
                }
            }
        }
        Ok(())
    }
}