use proc_macro::{*, Group as SG,TokenTree::{*}};
use std::sync::Mutex;
use core::{str::FromStr, ops::DerefMut};
static FUNCS: Mutex<Vec<(String,usize)>> = Mutex::new(Vec::new());

#[derive(Default)]
struct Flag(
    // TODO.
);
impl Flag {
    fn new()->Self{Default::default()}
    fn flag(&mut self, flag:String){
        todo!("{flag} is not a legal switch")
    }
}
fn add<T:Into<TokenTree>+Clone>(a:&mut TokenStream, b:&T){
    a.extend(<TokenTree as Into<TokenStream>>::into((b.clone()).into()).into_iter())
}
#[proc_macro_attribute]
pub fn export(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr=attr.into_iter().collect::<Vec<_>>();
    let mut flags=Flag::new();
    let mut keep = Vec::new();
    if attr.len()>0 {
        attr.split(|x|if let Punct(y)=x {y.as_char()==','} else {false}).for_each(|x|{
            if x.len() == 0 {
                println!("warning: ignore empty attribute.")
            } else if x.len() == 1 {
                flags.flag(x[0].to_string())
            } else {
                let key = x[0].to_string();
                match key.as_str() {
                    "keep" => keep.extend(x[2..].iter().map(|x|x.to_string())),
                    _=> println!("unknown key {}",x[0])
                }
            }
        })
    }
    keep.sort_unstable();
    println!("attr is {keep:?}");

    // TODO: macro attrs
    // keep=(var1,var2,..)
    // if parsed.len() > 0 {
    //     println!("warning: unknown option keys: [{}]", parsed.into_keys().collect::<Vec<_>>().as_slice().join(", "))
    // }
    let mut ret = TokenStream::from_str("#[inline(always)]").expect("FATAL, quote system does not function.");
    let mut iter = item.into_iter();
    while let Some(x) = iter.next(){
        add(&mut ret, &x);
        println!("x = {x:?}");
        if let Ident(_) = x{
            if x.to_string() == "fn" {
                break
            }
        }
    }

    // ..... fn @ name (params) -> out {...}

    let (fname,safe_name,unsafe_name) = if let Ident(name) = iter.next().expect("need a name after fn") {
        // fn name
        let fname = name.to_string();
        add(&mut ret, &name);
        let name = fname.strip_prefix("r#").unwrap_or(&fname);
        (fname, format!("wrap_{name}_safe"), format!("wrap_{name}_unsafe"))
    } else {
        panic!("need a name after fn");
    };

    // ... name @ (params) -> out {...}
    let group = if let Some(Group(g)) = iter.next(){
        let mut gparam=g.stream().into_iter().collect::<Vec<_>>().split(|x|if let Punct(y)=x {y.as_char()==','} else {false}).map(|x|{
            let [name, ty]=x.splitn(2,|x|if let Punct(y)=x {y.as_char()==':'} else {false}).collect::<Vec<_>>()[..] else { panic! ("should have the form `arg: ty`")};
// TODO: add parameter convertion in v0.5.0 (might with optional features)
//             if let Ok(_)=keep.binary_search(&name.to_string()) {
//
//             }
            x//.into_iter().cloned().into()
        }).collect::<Vec<TokenStream>>();
        // add(&mut ret, &Group::new(Delimiter::Parenthesis, gparam.iter().cloned().into())); //do it later.
        gparam
    } else {
        panic!("need a parameter list after name");
    };
    let mut grp: TokenStream = SG::new(Delimiter::Parenthesis, group.iter().cloned().into()).into();

    let fin:SG;
    while let Some(i) = iter.next() {
        if let Group(gg)=i {
            if gg.delimiter() == Delimiter::Brace {
                fin=gg;
                break
            }
        }
        add(&mut grp, &i)
    }

    ret.extend(grp.clone().into_iter());
    ret.extend(fin);

    println!("got grp = {}",grp.to_string());

    if let Some(wtf) = iter.next() {
        add(&mut ret, &wtf);
        ret.extend(iter);
        println!("warning, function have the form `fn(..)... {{...}} (unexpected more things)`, keep as-is.");
    }

    {
        let full=FUNCS.lock().expect("fatal error: internal errors while writting static variable FUNCS, compile again might help, file an issue might also help.").deref_mut();
        full.push((safe_name, group.len()));
        full.push((unsafe_name, group.len()));
    }

    ret
}
#[proc_macro]
pub fn done(input: TokenStream) -> TokenStream {
    let crate_name = if let Some(Ident(x)) = input.into_iter().next() {
        x.to_string()
    } else {
        println!("should provide a crate name.");
        return Default::default()
    };
    let data = core::mem::take(FUNCS.lock().expect("fatal error: internal errors while reading static variable FUNCS, compile again might help, file an issue might also help.").deref_mut());
    if data.len()==0 {
        println!("warning: no fn could be done, abort processing.");
        return Default::default();
    }
    let dlls=data.iter().map(|(name, cntr)|format!(r#"        ::rmin::reg::R_CallMethodDef {{name:"{name}\0".as_ptr() as *const _, fun:{name} as *const _, numArgs:{cntr}}},
"#)).collect::<String>();
    let fns=data.iter().map(|(name, cntr)|format!(r#"        fn {name}({parameters})->rmin::Owned<()>;
"#, parameters = (0..*cntr).map(|x|format!("arg{x}: Sexp<()>")).collect::<Vec<_>>().as_slice().join(", "))).collect::<String>();
    let s=format!(r#"mod {mod_name} {{
    pub extern "C" {{
{funcs}    }}
    const R_CALL_METHOD:&[::rmin::reg::R_CallMethodDef]=&[
{saves}        ::rmin::reg::R_CallMethodDef {{name: ::core::ptr::null(), fun: ::core::ptr::null(), numArgs:0}}
    ];

    #[no_mangle]
    extern fn R_init_{name}(info:*mut ::rmin::reg::DllInfo){{
        unsafe {{
            let res=::rmin::reg::R_registerRoutines(
                info,
                ::core::ptr::null(),
                R_CALL_METHOD.as_ptr(),
                ::core::ptr::null(),
                ::core::ptr::null()
            );
            let dynres = ::rmin::reg::R_useDynamicSymbols(info, 0);
            let force = ::rmin::reg::R_forceSymbols(info, 1);
        }}
    }}
}}"#,name=crate_name,saves=dlls, funcs=fns, mod_name="_please_do_not_use_rmin_export_interface_as_your_mod_name_");
    println!("finalizer generates:\n{s}");
    TokenStream::from_str(&s).expect("fatal error: internal errors with macro `done`, please disable the `done` macro, and file an issue about that.")
}
