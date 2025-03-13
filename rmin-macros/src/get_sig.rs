use crate::*;
/// before: ... name @ (params) -> out {...}
/// after : ... name (params) -> out @ {...} // finding the first {...}
pub fn get_sig(
    ret: &mut TokenStream,
    iter: &mut impl Iterator<Item = TokenTree>,
) -> (String, Vec<[String; 2]>) {
    let Some(Group(g)) = iter.next() else {
        panic!("need a parameter list after name");
    };
    let gparam = g
        .stream()
        .into_iter()
        .collect::<Vec<_>>()
        .split(|x| {
            if let Punct(y) = x {
                y.as_char() == ','
            } else {
                false
            }
        })
        .filter_map(|x| {
            let def = x
                .splitn(2, |x| {
                    if let Punct(y) = x {
                        y.as_char() == ':'
                    } else {
                        false
                    }
                })
                .collect::<Vec<_>>();
            if let [name, ty] = def[..] {
                Some([
                    TokenStream::from_iter(name.iter().cloned()).to_string(),
                    TokenStream::from_iter(ty.iter().cloned()).to_string(),
                ])
            } else {
                #[cfg(feature = "warning-on-empty-sig")]
                println!("warning, {def:?} is omitted (maybe a function with empty signature)");
                None
            }
        })
        .collect::<Vec<[String; 2]>>();

    let mut grp: TokenStream = From::<TokenTree>::from(g.into());

    while let Some(i) = iter.next() {
        if let Group(gg) = i {
            if gg.delimiter() == Delimiter::Brace {
                ret.extend(grp.clone().into_iter());
                add(ret, &gg);
                return (grp.to_string(), gparam);
            }
        } else {
            add(&mut grp, &i)
        }
    }
    panic!("need a group with `Delimiter::Brace`, found none.")
}
