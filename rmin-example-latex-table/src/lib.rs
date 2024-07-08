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

use rmin::*;
use std::str;


// R example:
// .Call('_rust_print_wrapper__0', c(1,2,3,4,.05,.06), c("1","2"), c("cap","label"), c("l|","cc|c%"), c("r1","r2","cp"),4L)
#[export]
fn print(
    data:Sexp<f64>, // data
    col_name:Sexp<character>, // data name
    caption:Sexp<character>, // with label
    rows: Sexp<character>, // template data
    rows_names: Sexp<character>, // template data
    roundings: Sexp<integer>, // roundings
) -> Owned<character> {
    let mut a = Table::default();
    let cap = caption.data();
    let rows = rows.data();
    let roundings = roundings.data();
    let rows = if rows.len() < 2 {
        panic!("Should provide row format for both template and data respectively. Instead, current provided row format has length {}", rows.len())
    } else {
        [rows[0], rows[1]].map(|x|str::from_utf8(x.data()).expect("the given row format could not be converted into utf8").to_string())
    };
    let rows_names : Vec<_> = rows_names.data().iter().map(|x|str::from_utf8(x.data()).expect("the given row name could not be converted into utf8").to_string()).collect();
    a.caption.cap = cap.get(0).map(|caption|str::from_utf8(caption.data()).ok()).flatten().map(|x|x.to_string());
    a.caption.label = cap.get(1).map(|label|str::from_utf8(label.data()).ok()).flatten().map(|x|x.to_string());


    a.content = TableContent::new();
    a.content.template_rows = Rows::header(&rows[0]);
    a.content.data_rows = Rows::header(&rows[1]);
    if roundings.len()==1 {
        a.content.data_rows.set_roundings(roundings[0] as u32);
    } else if roundings.len()==rows_names.len() {
        a.content.data_rows.rows.iter_mut().zip(roundings.iter()).for_each(|(x,&y)|x.ifn.rounding=y as u32)
    }
    a.content.data_rows.set_names(rows_names);
    
    a.content.data = data.data().iter().cloned().collect();
    a.content.col_name = col_name.data().iter().map(|x|String::from_utf8_lossy(x.data()).to_string()).collect();
    let out = a.to_string();
    println!("{out}");
    Owned::raw_from_str(out.as_bytes())
}

#[cfg(test)]
mod test {
    use crate::*;
    #[test]
    fn main() {
        let mut a = Table::default();
        a.set_caption("fine");
        a.set_label("t1");
        a.content = TableContent::new();
        a.content.template_rows = Rows::header("ll|");
        a.content.data_rows = Rows::header("ccc%|ccc%|ccc%");
        a.content.data_rows.set_names(vec![
            "bias".to_string(),
            "RMSE".to_string(),
            "CP".to_string(),
            "bias".to_string(),
            "RMSE".to_string(),
            "CP".to_string(),
            "bias".to_string(),
            "RMSE".to_string(),
            "CP".to_string(),
        ]);
        a.content.data = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
        a.content.data_rows.set_roundings(4u32);
        a.content.col_name = vec!["& test".to_string()];
        println!("{a}")
    }
}
