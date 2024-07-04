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
    a.content.col_name = Rows::header("ccc|ccc|ccc");
    println!("{a}")
}
