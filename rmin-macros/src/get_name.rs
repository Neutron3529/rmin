use crate::*;
/// before: @ ..... fn name (params) -> out {...}
/// after : ..... fn name @ (params) -> out {...} // finding the first fn and read name
pub fn get_name(ret:&mut TokenStream, iter:&mut impl Iterator<Item=TokenTree>)->String {
    while let Some(x) = iter.next(){
        add(ret, &x);
        // println!("x = {x:?}");
        if let Ident(_) = x{
            if x.to_string() == "fn" {
                break
            }
        }
    }

    if let Ident(name) = iter.next().expect("need a name after fn") {
        let fname = name.to_string();
        add(ret, &name);
        fname
    } else {
        panic!("need a name after fn");
    }
}
