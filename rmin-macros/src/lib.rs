use proc_macro::{*,TokenTree::{*}};
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
        keep.sort_unstable();
        println!("attr is {keep:?}");
    }

    let mut ret = TokenStream::from_str("#[inline(always)]").expect("FATAL, quote system does not function.");
    let mut iter = item.into_iter();

    // before: @ ..... fn name (params) -> out {...}
    let fname = get_name(&mut ret, &mut iter);
    // after : ..... fn name @ (params) -> out {...} // finding the first fn and read name
    let name = fname.strip_prefix("r#").unwrap_or(&fname);
    let (safe_name,unsafe_name) = (format!("wrap_{name}_safe"), format!("wrap_{name}_unsafe"));

    // before: ... name @ (params) -> out {...}
    let (sig,gparam) = get_sig(&mut ret, &mut iter);
    // after : ... name (params) -> out @ {...} // finding the first {...}

    println!("got name = {name}, sig = `{sig}`, gparam = {gparam:?}");

    if let Some(wtf) = iter.next() {
        add(&mut ret, &wtf);
        ret.extend(iter);
        println!("warning, function have the form `fn(..)... {{...}} (unexpected more things)`, keep {wtf:?} as-is.");
    }

    {
        let mut lck=FUNCS.lock().expect("fatal error: internal errors while writting static variable FUNCS, compile again might help, file an issue might also help.");
        let full = lck.deref_mut();
        full.push((safe_name, gparam.len()));
        full.push((unsafe_name, gparam.len()));
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
    extern "C" {{
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
