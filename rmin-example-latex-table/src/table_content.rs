use crate::{fmt, Columns, Data, Display};
/// Table content
///     \begin{tabular} % defined in Table
///         \toprules % defined in Table
///             % row_name[0] .. row_name[last]    template_columns[0] .. template_columns[last]
///             meta                            &
///         \bottomrules % defined in Table
///     \end{tabular} % defined in Table
#[derive(Default)]
pub struct TableContent {
    /// real data
    pub data: Vec<Data<f64>>,
    // /// real data: data cols
    // pub data_cols:usize,
    /// col template
    pub data_columns: Columns,
    /// col names, `self.cols()` equals to `self.row_name.len()`.
    pub row_name: Vec<String>,
    /// store meta informations by column-first storage.
    pub meta: Option<String>,
    /// column names
    pub template_columns: Columns,
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
        format!("{}{}", self.template_columns, self.data_columns)
    }
    pub fn set_data<T: Into<Vec<f64>> + Sized, U: Into<Vec<String>> + Sized>(
        &mut self,
        data: T,
        row_name: U,
    ) {
        self.data = Data::from(&data.into());
        self.row_name = row_name.into();
    }
    pub fn cols(&self) -> usize {
        self.row_name.len()
    }
}
impl Display for TableContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self
            .template_columns
            .names
            .iter()
            .chain(self.data_columns.names.iter());
        if let Some(item) = iter.next_back() {
            write!(f, "\n        ")?;
            for item in iter {
                write!(f, "{item} & ")?
            }
            write!(f, r"{item}\\")?
        }
        let col_len = self.row_name.len();
        let column_len = self.data_columns.names.len();
        if col_len * column_len != self.data.len() {
            write!(
                f,
                "%<Error: template accept {col_len} x {column_len} matrix, but only {} provided>",
                self.data.len()
            )?;
        } else {
            let mut data = self.data.clone();

            for j in 0..column_len {
                let r = &self.data_columns.columns[j];
                for i in 0..col_len {
                    data[j * col_len + i].rounding = r.ifn.rounding;
                    data[j * col_len + i].as_percentage = r.ifn.as_percentage;
                }
            }
            let mut hidx = self.hline.iter().copied();
            let mut cidx = self.cline.iter().copied();
            let mut chidx = hidx.next();
            let mut ccidx = cidx.next();
            for i in 0..col_len {
                while let Some(x) = chidx {
                    if x <= i {
                        write!(f, "\n        \\hline")?;
                        chidx = hidx.next();
                    } else {
                        break;
                    }
                }
                while let Some((x, start, end)) = ccidx {
                    if x <= i {
                        write!(f, "\n        \\cline{{{start}-{end}}}")?;
                        ccidx = cidx.next();
                    } else {
                        break;
                    }
                }
                if self.template_columns.columns.len() > 0 {
                    let colname = &self.row_name[i];
                    if colname.contains(r"\multicolumn") {
                        write!(f, "\n        {} & ", colname)?
                    } else {
                        let less = colname.bytes().fold(
                            self.template_columns.columns.len() as i32,
                            |s, x| if x == b'&' { s - 1 } else { s },
                        );
                        if less < 0 {
                            panic!("{} more `&`{s} is provided in colname {colname}",-less, s=if less==-1 {""} else {"s"})
                        } else if less == 0 {
                            write!(f, "\n        {} ", self.row_name[i])?
                        } else {
                            if less>1 && self.template_columns.columns.first().map(|x|x.align!=crate::ColumnAlign::Left).unwrap_or(true){
                                write!(f, "\n        {:&>width$} {colname} & ", "", width = (less-1) as usize)?
                            } else {
                                write!(f, "\n        {colname} {:&>width$} ", "", width = less as usize)?
                            }
                        }
                    }
                } else {
                    write!(f, "\n        ")?
                }
                for j in 0..column_len {
                    write!(f, "{}", data[i + j * col_len])?;
                    if j != column_len - 1 {
                        write!(f, " & ")?
                    } else {
                        write!(f, r"\\")?
                    }
                }
            }
            while let Some((x, start, end)) = ccidx {
                if x <= col_len {
                    write!(f, "\n        \\cline{{{start}-{end}}}")?;
                    ccidx = cidx.next();
                } else {
                    break;
                }
            }
            while let Some(x) = chidx {
                if x <= col_len {
                    write!(f, "\n        \\hline")?;
                    chidx = hidx.next();
                } else {
                    break;
                }
            }
        }
        Ok(())
    }
}
