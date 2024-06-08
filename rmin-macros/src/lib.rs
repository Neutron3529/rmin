use proc_macro::{*,TokenTree::{*}};
use std::sync::Mutex;
use core::{str::FromStr, ops::DerefMut};
static FUNCS: Mutex<Vec<Meta>> = Mutex::new(Vec::new());

#[derive(Default)]
struct Flag(
    // TODO.
);
#[derive(Clone)]
struct Meta {
    fname:String,
    params:Vec<[String;2]>
}
impl Meta {
    fn name(&self) -> &str {
        self.fname.strip_prefix("r#").unwrap_or(&self.fname)
    }
    fn safe_name(&self) -> String {
        #[cfg(feature = "camel-ass")]
        format!{"_rUST_{}_wRAPPER_" , self.name()}
        #[cfg(not(feature = "camel-ass"))]
        format!("_rust_{}_wrapper_" , self.name())
    }
    fn unsafe_name(&self) -> String {
        #[cfg(feature = "camel-ass")]
        format!{"_rUST_{}_wRAPPER_uNSAFE_" , self.name()}
        #[cfg(not(feature = "camel-ass"))]
        format!("_rust_{}_wrapper_unsafe_" , self.name())
    }
    fn param(&self) -> String {
        self.params.iter().map(|x|x[0].clone()).collect::<Vec<_>>().join(", ")
    }
    fn param_check(&self, delim:&str) -> String {
        self.params.iter().map(|x|format!("{}.missing()",x[0])).collect::<Vec<_>>().join(delim)
    }
    fn param_check_report(&self) -> String {
        self.params.iter().map(|x|format!("  missing {}: {{}}",x[0])).collect::<Vec<_>>().join("\n")
    }
}

impl Flag {
    fn new()->Self{Default::default()}
    fn flag(&mut self, flag:String){
        todo!("{flag} is not a legal switch")
    }
}
fn add<T:Into<TokenTree>+Clone>(a:&mut TokenStream, b:&T){
    a.extend(<TokenTree as Into<TokenStream>>::into((b.clone()).into()).into_iter())
}


mod get_name;
use get_name::get_name;
mod get_sig;
use get_sig::get_sig;

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
        });

        // TODO: parse macro attrs
        // if parsed.len() > 0 {
        //     println!("warning: unknown option keys: [{}]", parsed.into_keys().collect::<Vec<_>>().as_slice().join(", "))
        // }
        // keep.sort_unstable();
        println!("attr is {keep:?}");
    }

    let mut ret = TokenStream::from_str("#[inline(always)]").expect("FATAL, quote system does not function.");
    let mut iter = item.into_iter();

    // before: @ ..... fn name (params) -> out {...}
    let fname = get_name(&mut ret, &mut iter);
    // after : ..... fn name @ (params) -> out {...} // finding the first fn and read name
    // before:      ... name @ (params) -> out {...}
    let (sig,params) = get_sig(&mut ret, &mut iter);
    // after :      ... name (params) -> out @ {...} // finding the first {...}
    #[cfg(feature = "verbose-output")]
    println!("got fn_name = {fname}, sig = `{sig}`, params = {params:?}");

    if let Some(wtf) = iter.next() {
        add(&mut ret, &wtf);
        ret.extend(iter);
        println!("warning, function have the form `fn(..)... {{...}} (unexpected more things)`, keep {wtf:?} as-is.");
    }

    let meta = Meta {
        fname,
        params
    };

    let n = {
        let mut vec = FUNCS.lock()
            .expect("fatal error: internal errors while writting static variable FUNCS, compile again might help, file an issue might also help.");
        let len = vec.deref_mut().len();
        vec.deref_mut().push(meta.clone());
        len
    };

    let fname = &meta.fname;
    let safe = meta.safe_name();
    let usafe = meta.unsafe_name();
    let param = meta.param();
    let check = meta.param_check(" || ");
    let check_vals = meta.param_check(".missing(), ");
    let report = meta.param_check_report();
    let safe_variant = if check.len()>0 {format!(r#"
            if {check} {{
                rmin::handle_panic(||panic!("Parameter missing detected\n{report}", {check_vals}.missing()))
            }} else {{
                {usafe}_{n}({param})
            }}"#) } else {format!(r#"
            {usafe}_{n}({param})"#)};

    let expanded = format!(r#"
    mod {fname} {{
        use super::*;
        #[no_mangle]
        extern fn {safe}_{n} {sig} {{{safe_variant}
        }}
        #[no_mangle]
        extern fn {usafe}_{n} {sig} {{
            rmin::handle_panic(||{fname}({param}))
        }}
    }}"#);
    #[cfg(feature = "verbose-output")]
    println!("#[export] writting additional mod: {expanded}");
    let res = TokenStream::from_str(&expanded).unwrap_or_else(|err|panic!("macro auto expand to {expanded}, there should be an unexpected error: {err:?}. File an issue please."));
    ret.extend(res);
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
    // data.sort_unstable_by(|a,b|a.name.cmp(&b.name));
    let iter=data.iter().enumerate().map(|(n,x)|(n as isize,x.safe_name(),x.params.len())).chain(data.iter().enumerate().map(|(n,x)|(!(n as isize),x.unsafe_name(),x.params.len())));
    let dlls=iter.clone().map(|(n,name, cntr)|format!(r#"        R_CallMethodDef {{name:c".{prefix}{cname}".as_ptr(), fun:{name}_{n} as *const _, numArgs: {cntr}}},
"#,cname = if n<0{!n} else {n}, prefix = if n <0 {"u"} else {"c"}, n=if n<0 {!n} else {n})).collect::<String>();
    let fns=iter.clone().map(|(n,name, cntr)|format!(r#"        fn {name}_{n}({parameters})->Owned<()>;
"#, parameters = (0..cntr).map(|x|format!("arg{x}: Sexp<()>")).collect::<Vec<_>>().as_slice().join(", "), n=if n<0 {!n} else {n})).collect::<String>();
    let s=format!(r#"mod {mod_name} {{{camel}
    use ::rmin::{{Sexp, Owned, reg::*}};
    use ::core::ptr::null;
    extern "C" {{
{funcs}    }}
    const R_CALL_METHOD:&[R_CallMethodDef]=&[
{saves}        R_CallMethodDef {{name: null(), fun: null(), numArgs: 0}}
    ];

    #[no_mangle]
    extern fn R_init_{name}(info:*mut DllInfo){{
        unsafe {{
            R_registerRoutines(
                info,
                null(),
                R_CALL_METHOD.as_ptr(),
                null(),
                null()
            );
            R_useDynamicSymbols(info, 0);
            R_forceSymbols(info, 1); // change this to 1 will make most of the functions unsearchable, which is sad for people who want to compile in Rust and load in R directly.
        }}
    }}
}}"#,name=crate_name,saves=dlls, funcs=fns, mod_name="_please_do_not_use_rmin_export_interface_as_your_mod_name_", camel = if cfg!(feature = "camel-ass") {"\n"} else {""});
    #[cfg(feature = "verbose-output")]
    println!("finalizer generates:\n{s}");
    TokenStream::from_str(&s).expect("fatal error: internal errors with macro `done`, please disable the `done` macro, and file an issue about that.")
}
