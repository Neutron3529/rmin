//use rmin::*;
use core::fmt::{self, Display};

mod table;
use table::Rules;
pub use table::Table;

mod caption;
pub use caption::Caption;

mod column;
pub use column::{Column, ColumnAlign, Columns, ItemFn};

mod data;
pub use data::Data;

mod table_content;
pub use table_content::TableContent;

use rmin::{*, println};
use std::{str, string::String};

/// R example:
/// .Call('_rust_print_wrapper__0', c(1,2,3,4,.05,.06), c("1","2"), logical(0), logical(0), integer(0),c("cap","label"), c("l|","cc|c%"), c("r1","r2","cp"),4L)
#[export]
fn _print(
    data: Sexp<f64>,                   // data
    row_name: Sexp<character>,         // data name
    hline: Sexp<integer>,              // hline
    cline: Sexp<integer>,              // cline
    bold: Sexp<logical>,               // whether the data should be bolded
    italic: Sexp<logical>,             // whether the data should be italic
    stars: Sexp<integer>,              // how much stars the data should have
    caption: Sexp<character>,          // with label
    columns: Sexp<character>,          // template data
    column_names: Sexp<character>,     // template data
    roundings: Sexp<integer>,          // roundings
    table_rules: Sexp<character>,      // template data
    top_rules: Sexp<character>,        // template data
    bottom_rules: Sexp<character>,     // template data
    footnotes: Sexp<character>,        // template data
    use_threeparttable: Sexp<logical>, // indicate whether threeparttable is used.
) -> Owned<character> {
    let mut a = Table::default();
    let cap = caption.data();
    let columns = columns.data();
    let roundings = roundings.data();
    let columns = if columns.len() < 2 {
        panic!("Should provide column format for both template and data respectively. Instead, current provided column format has length {}", columns.len())
    } else {
        [columns[0], columns[1]].map(|x| {
            str::from_utf8(x.data())
                .expect("the given column format could not be converted into utf8")
                .to_string()
        })
    };
    let column_names: Vec<_> = column_names
        .data()
        .iter()
        .map(|x| {
            str::from_utf8(x.data())
                .expect("the given column name could not be converted into utf8")
                .to_string()
        })
        .collect();
    a.caption.cap = cap
        .get(0)
        .map(|caption| str::from_utf8(caption.data()).ok())
        .flatten()
        .map(|x| x.to_string());
    a.caption.label = cap
        .get(1)
        .map(|label| str::from_utf8(label.data()).ok())
        .flatten()
        .map(|x| x.to_string());

    a.content = TableContent::new();
    a.content.template_columns = Columns::header(&columns[0]);
    a.content.data_columns = Columns::header(&columns[1]);
    if roundings.len() == 1 {
        a.content.data_columns.apply_default_roundings(roundings[0]);
    } else if roundings.len() == column_names.len() {
        a.content
            .data_columns
            .apply_roundings(roundings.iter().copied())
    } else {
        panic!("specific roundings does not equals to 1 or data columns")
    }
    a.content.data_columns.set_names(column_names);

    a.content.data = Data::from(data.data().iter());
    fn modify<'a, T: 'a>(
        a: &'a mut Table,
        format: impl ExactSizeIterator<Item = &'a T>,
        modify: impl FnMut((&'a mut Data<f64>, &'a T)),
    ) {
        if format.len() > 0 {
            if format.len() != a.content.data.len() {
                panic!(
                    "format length {} is not equals to data length {}",
                    format.len(),
                    a.content.data.len()
                )
            } else {
                a.content.data.iter_mut().zip(format).for_each(modify)
            }
        }
    }
    modify(&mut a, bold.data().iter(), |(data, &item)| {
        data.as_bold = item == 1
    });
    modify(&mut a, italic.data().iter(), |(data, &item)| {
        data.as_italic = item == 1
    });
    modify(&mut a, stars.data().iter(), |(data, &item)| {
        data.stars = item
    });
    if let Some(&1) = use_threeparttable.data().get(0) {
        a.content
            .data
            .iter_mut()
            .for_each(|x| x.use_threeparttable = true)
    }

    a.content.hline = hline.data().iter().map(|&x| x as usize).collect();
    a.content.cline = cline
        .data()
        .chunks(3)
        .map(|x| (x[0] as usize, x[1] as usize, x[2] as usize))
        .collect();
    a.content.hline.sort_unstable();
    a.content.cline.sort_unstable();
    a.content.row_name = row_name
        .data()
        .iter()
        .map(|x| String::from_utf8_lossy(x.data()).to_string())
        .collect();

    a.table_rules = Rules::new(
        table_rules
            .data()
            .iter()
            .map(|x| String::from_utf8_lossy(x.data()).to_string())
            .collect(),
    );
    a.top_rules = Rules::new(
        top_rules
            .data()
            .iter()
            .map(|x| String::from_utf8_lossy(x.data()).to_string())
            .collect(),
    );
    a.bottom_rules = Rules::new(
        bottom_rules
            .data()
            .iter()
            .map(|x| String::from_utf8_lossy(x.data()).to_string())
            .collect(),
    );
    a.footnotes = Rules::new(
        footnotes
            .data()
            .iter()
            .map(|x| String::from_utf8_lossy(x.data()).to_string())
            .collect(),
    );

    let out = a.to_string().replace('%', r"\%");
    println!("{out}");
    Owned::raw_from_str(out.as_bytes())
}

#[cfg(test)]
mod test {
    use crate::*;
    use std::println; // since this is test, we do not load R environment
    #[test]
    fn main() {
        let mut a = Table::default();
        a.set_caption("fine");
        a.set_label("t1");
        a.content = TableContent::new();
        a.content.template_columns = Columns::header("ll|");
        a.content.data_columns = Columns::header("ccc%|ccc%|ccc%");
        a.content.data_columns.set_names(vec![
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
        a.content.data = Data::from(&vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9]);
        a.content.data_columns.set_roundings(4);
        a.content.row_name = vec!["& test".to_string()];
        println!("{a}")
    }
}
done!();
