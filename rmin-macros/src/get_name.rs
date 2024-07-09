use crate::*;
/// before: @ ..... fn name (params) -> out {...}
/// after : ..... fn name @ (params) -> out {...} // finding the first fn and read name
pub fn get_meta(ret: &mut TokenStream, iter: &mut impl Iterator<Item = TokenTree>) -> crate::Meta {
    #[cfg(feature = "write-r-func-to-out-dir")]
    let mut meta = String::new();
    #[cfg(not(feature = "write-r-func-to-out-dir"))]
    let meta = String::new();
    #[cfg(feature = "write-r-func-to-out-dir")]
    let mut prev_is_sharp = false;
    #[cfg(feature = "write-r-func-to-out-dir")]
    let mut public = false;
    while let Some(x) = iter.next() {
        add(ret, &x);
        // println!("x = {x:?}");
        if let Ident(_) = x {
            if x.to_string() == "fn" {
                break
            }
            #[cfg(feature = "write-r-func-to-out-dir")]
            if x.to_string() == "pub" {
                public = true
            }
        } else {
            #[cfg(feature = "write-r-func-to-out-dir")]
            parse_doc(&mut meta, &mut prev_is_sharp, x);
        }
    }

    if let Ident(name) = iter.next().expect("need a name after fn") {
        let fname = name.to_string();
        add(ret, &name);
        Meta {
            params: vec![[fname, meta]],
            #[cfg(feature = "write-r-func-to-out-dir")]
            public
        }
    } else {
        panic!("need a name after fn");
    }
}

#[cfg(feature = "write-r-func-to-out-dir")]
fn parse_doc(meta: &mut String, prev_is_sharp: &mut bool, x: TokenTree) {
    // status: ..@.. @ means I'm here.
    if *prev_is_sharp {
        // status: # @..
        *prev_is_sharp = false;
        // check whether the current token is `[...]`
        let Group(x) = x else { return };
        if x.delimiter() != Delimiter::Bracket {
            return;
        }
        // status: # @[...]

        if let [Ident(ref ident), Punct(ref eq), Literal(ref lit)] =
            x.stream().into_iter().collect::<Vec<_>>()[..]
        {
            if ident.to_string() == "doc" && eq.as_char() == '=' {
                meta.push_str(&format!(
                    "\n#' {}",
                    lit.to_string()
                        .trim_matches(|c| c == '\"' || c == '\'')
                        .replace(r#"\'"#, r#"'"#)
                        .replace(r#"\""#, r#"""#)
                        .replace(r#"\n"#, "\n#' ")
                        .replace(r#"\t"#, "\t")
                        .replace(r#"\r"#, "\r")
                ));
            }
        }

        // let stream = x.stream().into_iter().collect::<Vec<_>>();
        // // status: # @[stream]
        // if stream.len()<3 { return } // cannot satisfy doc = literal
        // let Ident(ref ident) = stream[0] else { return };
        // if ident.to_string() != "doc" { return }
        // // status: # @[doc stream[1..]]
        // let Punct(ref ch) = stream[1] else { return };
        // if ch.as_char() != '=' { return }
        // // status: # @[doc = stream[2..]]
        // let Literal(ref lit) = stream[2] else { return };
        // meta.push_str(&format!("\n#' {}", lit.to_string().trim_matches(|c| c == '\"' || c == '\'').replace(r#"\'"#,r#"'"#).replace(r#"\""#,r#"""#).replace(r#"\n"#,"\n#' ").replace(r#"\t"#,"\t").replace(r#"\r"#,"\r") ));
    } else if let Punct(x) = x {
        if x.as_char() == '#' {
            *prev_is_sharp = true;
        }
    }
}
