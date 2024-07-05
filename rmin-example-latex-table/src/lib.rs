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

mod table_content;
pub use table_content::TableContent;

fn main() {
    let mut a = Table::default();
    a.set_caption("fine");
    a.set_label("t1");
    a.content = TableContent::new();
    a.content.template_rows = Rows::header("ll|");
    a.content.data_rows = Rows::header("ccc%|ccc%|ccc%");
    a.content.data_rows.set_names(vec!["bias".to_string(), "RMSE".to_string(), "CP".to_string(),"bias".to_string(), "RMSE".to_string(), "CP".to_string(),"bias".to_string(), "RMSE".to_string(), "CP".to_string()]);
    a.content.data=vec![0.1,0.2,0.3,0.4,0.5,0.6,0.7,0.8,0.9];
    a.content.data_rows.set_roundings(4u32);
    a.content.col_name = vec!["& test".to_string()];
    println!("{a}")
}
