//use rmin::*;
use core::fmt::{self, Display};

mod table;
pub use table::Table;

mod caption;
pub use caption::Caption;

mod row;
pub use row::{ItemFn, Row, RowAlign, Rows};

mod data;
pub use data::Data;

#[derive(Default)]
pub struct TableContent {
    pub template_rows: Rows,
    pub col_name: Rows,
}
impl TableContent {
    fn new() -> Self {
        Default::default()
    }
    fn format(&self) -> String {
        format!("{}{}", self.template_rows, self.col_name)
    }
}
impl Display for TableContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        drop(f);
        Ok(())
    }
}
// pub struct TableContent{
//     pub format: Format,
//     pub data:Vec<f64>,
//     pub data_cols:usize,
//     pub col_name:Vec<String>,
//     pub header:Vec<String>,
//     pub hline:Vec<usize>,
//     pub cline:Vec<(usize,usize,usize)>,
// }
// impl Display for TableContent {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         for i in self.header {
//             write!(f,"{i}")?
//         }
//         let data_rows = self.data.len()/self.data_cols;
//         if data_rows != self.format.row.len() {
//             write!(f,"<Error: data row count does not matches the template row count.>")?;
//         } else if self.col_name.len() != self.data_cols {
//             write!(f,"<Error: data column count does not matches the template column count.>")?;
//         } else {
//             for i in 0..self.data_cols {
//                 write!(f, "        {} & ",col_name[i])?;
//                 for j in 0..data_rows {
//                     let data = self.data[i+j*self.data_cols];
//                     self.format.row[j].format
//                     if j != data_rows - 1 {
//                         write!(f, " & ")?
//                     } else {
//                         write!(f, "\\\\\n")?
//                     }
//                 }
//             }
//         }
//         Ok(())
//     }
// }
//
// pub struct Format {
//     pub data:Vec<Row>,
//     pub row:Vec<Row>,
// }
// impl Display for Format {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         for r in self.data.iter().chain(self.row.iter()) {
//             write!(f, "{r}")?
//         }
//         Ok(())
//     }
// }

fn main() {
    let mut a = Table::default();
    a.caption = Caption {
        cap: Some("fine".to_string()),
        label: Some("t1".to_string()),
    };
    a.content = TableContent::new();
    a.content.template_rows = Rows::header("ll|");
    a.content.col_name = Rows::header("ccc|ccc|ccc");
    println!("{a}")
}
