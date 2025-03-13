use core::{ops::DerefMut, str::FromStr};
use proc_macro::{TokenTree::*, *};
use std::sync::Mutex;
static FUNCS: Mutex<Vec<Meta>> = Mutex::new(Vec::new());
#[cfg(feature = "write-r-func-to-out-dir")]
const R_SCRIPT_NAME: &str = "aaa.rmin.Rust.Functions.R";

#[derive(Default)]
struct Flag(
    // TODO.
);
#[derive(Clone)]
struct Meta {
    params: Vec<[String; 2]>,
    #[cfg(feature = "write-r-func-to-out-dir")]
    public: bool
}
impl Meta {
    fn fname(&self) -> &str {
        &self.params[0][0]
    }
    #[cfg(feature = "write-r-func-to-out-dir")]
    fn doc(&self) -> &str {
        &self.params[0][1]
    }
    fn name(&self) -> &str {
        self.fname().strip_prefix("r#").unwrap_or(self.fname())
    }
    fn safe_name(&self) -> String {
        #[cfg(feature = "camel-ass")]
        format! {"_rUST_{}_wRAPPER_" , self.name()}
        #[cfg(not(feature = "camel-ass"))]
        format!("_rust_{}_wrapper_", self.name())
    }
    fn unsafe_name(&self) -> String {
        #[cfg(feature = "camel-ass")]
        format! {"_rUST_{}_wRAPPER_uNSAFE_" , self.name()}
        #[cfg(not(feature = "camel-ass"))]
        format!("_rust_{}_wrapper_unsafe_", self.name())
    }
    fn param(&self) -> String {
        self.params
            .iter()
            .skip(1)
            .map(|x| x[0].clone())
            .collect::<Vec<_>>()
            .join(", ")
    }
    fn param_check(&self, delim: &str) -> String {
        self.params
            .iter()
            .skip(1)
            .filter(|x|!x[1].starts_with("Option"))
            .map(|x| format!("{}.missing()", x[0]))
            .collect::<Vec<_>>()
            .join(delim)
    }
    fn param_check_report(&self) -> String {
        self.params
            .iter()
            .skip(1)
            .filter(|x|!x[1].starts_with("Option"))
            .map(|x| format!("  missing {}: {{}}", x[0]))
            .collect::<Vec<_>>()
            .join("\n")
    }
    fn len(&self) -> usize {
        self.params.len().wrapping_sub(1)
    }
}

impl Flag {
    fn new() -> Self {
        Default::default()
    }
    fn flag(&mut self, flag: String) {
        todo!("{flag} is not a legal switch")
    }
}
fn add<T: Into<TokenTree> + Clone>(a: &mut TokenStream, b: &T) {
    a.extend(<TokenTree as Into<TokenStream>>::into((b.clone()).into()).into_iter())
}

mod get_name;
use get_name::get_meta;
mod get_sig;
use get_sig::get_sig;

#[proc_macro_attribute]
pub fn export(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = attr.into_iter().collect::<Vec<_>>();
    let mut flags = Flag::new();
    let mut keep = Vec::new();
    if attr.len() > 0 {
        attr.split(|x| {
            if let Punct(y) = x {
                y.as_char() == ','
            } else {
                false
            }
        })
        .for_each(|x| {
            if x.len() == 0 {
                println!("warning: ignore empty attribute.")
            } else if x.len() == 1 {
                flags.flag(x[0].to_string())
            } else {
                let key = x[0].to_string();
                match key.as_str() {
                    "keep" => keep.extend(x[2..].iter().map(|x| x.to_string())),
                    _ => println!("unknown key {}", x[0]),
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

    let mut ret =
        TokenStream::from_str("#[inline(always)]").expect("FATAL, quote system does not function.");
    let mut iter = item.into_iter();

    // before: @ ..... fn name (params) -> out {...}
    let mut meta = get_meta(&mut ret, &mut iter);
    // after : ..... fn name @ (params) -> out {...} // finding the first fn and read name
    // before:      ... name @ (params) -> out {...}
    let (sig, params) = get_sig(&mut ret, &mut iter);
    // after :      ... name (params) -> out @ {...} // finding the first {...}

    #[cfg(feature = "verbose-output")]
    println!(
        "got fn_name = {fname}, sig = `{sig}`, params = {params:?}",
        fname = meta.fname()
    );

    meta.params.extend(params);

    if let Some(wtf) = iter.next() {
        add(&mut ret, &wtf);
        ret.extend(iter);
        println!("warning, function have the form `fn(..)... {{...}} (unexpected more things)`, keep {wtf:?} as-is.");
    }

    let n = {
        let mut vec = FUNCS.lock()
            .expect("fatal error: internal errors while writting static variable FUNCS, compile again might help, file an issue might also help.");
        let len = vec.deref_mut().len();
        vec.deref_mut().push(meta.clone());
        len
    };

    let fname = meta.fname();
    let safe = meta.safe_name();
    let usafe = meta.unsafe_name();
    let param = meta.param();
    let check = meta.param_check(" || ");
    let check_vals = meta.param_check(", ");
    let report = meta.param_check_report();
    let safe_variant = if cfg!(not(feature = "force-symbol")) && check.len() > 0 {
        // with routine registered, missing parameter is not allowed. thus there is no need to check parameters here again.
        // currently safe and unsafe do the same things, but things could be changed in the future if some extra check should be done.
        format!(
            r#"
            if {check} {{
                rmin::handle_panic(||panic!("Parameter missing detected\n{report}", {check_vals}))
            }} else {{
                {usafe}_{n}({param})
            }}"#
        )
    } else {
        format!(
            r#"
            {usafe}_{n}({param})"#
        )
    };

    let expanded = format!(
        r#"
    mod {fname} {{
        use super::*;
        #[no_mangle]
        extern fn {safe}_{n} {sig} {{{safe_variant}
        }}
        #[no_mangle]
        extern fn {usafe}_{n} {sig} {{
            rmin::handle_panic(||{fname}({param}))
        }}
    }}"#
    );
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
    } else if let Ok(s) = std::env::var("CARGO_PKG_NAME") {
        s.to_string()
    } else {
        println!("Warning: CARGO_PKG_NAME is not set. should provide a crate name.");
        return Default::default();
    };

    finalize(crate_name)
}
fn finalize(crate_name: String) -> TokenStream {
    let data = core::mem::take(FUNCS.lock().expect("fatal error: internal errors while reading static variable FUNCS, compile again might help, file an issue might also help.").deref_mut());
    if data.len() == 0 {
        println!("warning: no fn could be done, abort processing.");
        return Default::default();
    }
    // data.sort_unstable_by(|a,b|a.name.cmp(&b.name));
    let iter = data
        .iter()
        .enumerate()
        .map(|(n, x)| (n as isize, x.safe_name(), x.len()))
        .chain(
            data.iter()
                .enumerate()
                .map(|(n, x)| (!(n as isize), x.unsafe_name(), x.len())),
        );
    let dlls=iter.clone().map(|(n,name, cntr)|format!(r#"        R_CallMethodDef {{name:c".{prefix}{cname}".as_ptr(), fun:{name}_{n} as *const _, numArgs: {cntr}}},
"#,cname = if n<0{!n} else {n}, prefix = if n <0 {"u"} else {"c"}, n=if n<0 {!n} else {n})).collect::<String>();
    let fns = iter
        .clone()
        .map(|(n, name, cntr)| {
            format!(
                r#"        fn {name}_{n}({parameters})->Owned<()>;
"#,
                parameters = (0..cntr)
                    .map(|x| format!("arg{x}: Sexp<()>"))
                    .collect::<Vec<_>>()
                    .as_slice()
                    .join(", "),
                n = if n < 0 { !n } else { n }
            )
        })
        .collect::<String>();
    let s = format!(
        r#"mod {mod_name} {{{camel}
    use ::rmin::{{Sexp, Owned, reg::*}};
    use ::core::ptr::null;
    extern "C" {{
{funcs}    }}
    const R_CALL_METHOD:&[R_CallMethodDef]=&[
{saves}        R_CallMethodDef {{name: null(), fun: null(), numArgs: 0}}
    ];
    // in case `lib{name}.so` is used.
    #[no_mangle]
    extern fn R_init_lib{name}(info:*mut DllInfo){{
        R_init_{name}(info)
    }}
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
            R_forceSymbols(info, {forceSymbols}); // change this to 1 will make most of the functions unsearchable, which is sad for people who want to compile in Rust and load in R directly.
        }}
    }}
}}"#,
        name = crate_name,
        saves = dlls,
        funcs = fns,
        forceSymbols = if cfg!(feature = "force-symbol") {1} else {0},
        mod_name = "_please_do_not_use_rmin_export_interface_as_your_mod_name_",
        camel = if cfg!(feature = "camel-ass") {
            "\n"
        } else {
            ""
        }
    );

    #[cfg(feature = "verbose-output")]
    println!("finalizer generates:\n{s}");

    #[cfg(feature = "write-r-func-to-out-dir")]
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        #[cfg(feature = "verbose-output")]
        println!("Writting R wrappers to {dir}");
        use std::fs;
        use std::path::Path;
        let path = Path::new(&dir).join("R");
        if path.is_dir() {
            fs::write(
                path.join(R_SCRIPT_NAME),
                format!(
                    r#"# nolint start
#' @name {crate_name}
#' @docType package
#' @usage NULL
#' @useDynLib {crate_name}, .registration = TRUE
"_PACKAGE"

{all_fns}

# nolint end
"#,
                    all_fns = data
                        .iter()
                        .enumerate()
                        .map(|(n, meta)| format!(
                            r#"{docs}{export}
{name} <- function({param}).Call(.c{n}{sep}{param})"#,
                            docs = meta.doc(),
                            name = if meta.name().starts_with("_") {format!("`{}`", meta.name())} else {meta.name().to_string()},
                            param = meta.param(),
                            sep = if meta.len() == 0 { "" } else { ", " },
                            export = if meta.public { "\n#' @export" } else { "" }
                        ))
                        .collect::<Vec<_>>()
                        .join("\n\n")
                ),
            )
            .unwrap_or_else(|_| {
                println!("warning: failed to write R wrapper file `{R_SCRIPT_NAME}`")
            });
        } else {
            println!("Warning: environment variable $(R_SCRIPT_DIR) is set but the path `{dir}/R` is not a dir!")
        }
    } else {
        println!(
            "warning: $(CARGO_MANIFEST_DIR) does not have a value, thus abort writting R wrappers."
        )
    }
    TokenStream::from_str(&s).expect("fatal error: internal errors with macro `done`, please disable the `done` macro, and file an issue about that.")
}
#[cfg(feature = "write-r-func-to-out-dir")]
#[allow(non_snake_case)]
fn print_env() {
    let CARGO_MANIFEST_DIR = std::env::var("CARGO_MANIFEST_DIR");
    let CARGO_PKG_VERSION = std::env::var("CARGO_PKG_VERSION");
    let CARGO_PKG_VERSION_MAJOR = std::env::var("CARGO_PKG_VERSION_MAJOR");
    let CARGO_PKG_VERSION_MINOR = std::env::var("CARGO_PKG_VERSION_MINOR");
    let CARGO_PKG_VERSION_PATCH = std::env::var("CARGO_PKG_VERSION_PATCH");
    let CARGO_PKG_VERSION_PRE = std::env::var("CARGO_PKG_VERSION_PRE");
    let CARGO_PKG_AUTHORS = std::env::var("CARGO_PKG_AUTHORS");
    let CARGO_PKG_NAME = std::env::var("CARGO_PKG_NAME");
    let CARGO_PKG_DESCRIPTION = std::env::var("CARGO_PKG_DESCRIPTION");
    let CARGO_PKG_HOMEPAGE = std::env::var("CARGO_PKG_HOMEPAGE");
    let CARGO_PKG_REPOSITORY = std::env::var("CARGO_PKG_REPOSITORY");
    let OUT_DIR = std::env::var("OUT_DIR");

    println!(
        r#"vars:
    CARGO_MANIFEST_DIR = {CARGO_MANIFEST_DIR:?}
    CARGO_PKG_VERSION = {CARGO_PKG_VERSION:?}
    CARGO_PKG_VERSION_MAJOR = {CARGO_PKG_VERSION_MAJOR:?}
    CARGO_PKG_VERSION_MINOR = {CARGO_PKG_VERSION_MINOR:?}
    CARGO_PKG_VERSION_PATCH = {CARGO_PKG_VERSION_PATCH:?}
    CARGO_PKG_VERSION_PRE = {CARGO_PKG_VERSION_PRE:?}
    CARGO_PKG_AUTHORS = {CARGO_PKG_AUTHORS:?}
    CARGO_PKG_NAME = {CARGO_PKG_NAME:?}
    CARGO_PKG_DESCRIPTION = {CARGO_PKG_DESCRIPTION:?}
    CARGO_PKG_HOMEPAGE = {CARGO_PKG_HOMEPAGE:?}
    CARGO_PKG_REPOSITORY = {CARGO_PKG_REPOSITORY:?}
    OUT_DIR = {OUT_DIR:?}"#
    )
}
