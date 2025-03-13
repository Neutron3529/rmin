use std::{
    env,
    ffi::OsString,
    io::{self, Error, ErrorKind},
    path::{Path, PathBuf},
    process::{exit, Command},
};

const ENVVAR_R_HOME: &str = "R_HOME";

// Get the path to the R home either from an envvar or by executing the actual R binary on PATH.
fn get_r_home() -> io::Result<PathBuf> {
    // If the environment variable R_HOME is set we use it
    if let Some(r_home) = env::var_os(ENVVAR_R_HOME) {
        return Ok(PathBuf::from(r_home));
    }

    // Otherwise, we try to execute `R` to find `R_HOME`. Note that this is
    // discouraged, see Section 1.6 of "Writing R Extensions"
    // https://cran.r-project.org/doc/manuals/r-release/R-exts.html#Writing-portable-packages
    let output = Command::new("R").arg("RHOME").output()?;
    if !output.stdout.is_empty() {
        let mut output=output.stdout;
        if let Some(b'\n') = output.last() {
            output.pop();
        } // pop \n
        if let Some(b'\r') = output.last() {
            output.pop();
        } // pop \r, thus the final `\r\n` is poped if exists.
        if output.len()>0 {
            // SAFETY: Obtain OsString from stdout with
            Ok(PathBuf::from(unsafe {OsString::from_encoded_bytes_unchecked(output)}))
        } else {
            Err(Error::new(ErrorKind::Other, "Cannot find R home with command `R RHOME`.\nNote: Specific `$R_HOME` variable directly could solve this issue.\n    The $R_HOME variable could be set automatically with `R CMD build` command if you are building an R package.\n    R founded in $PATH, but cannot obtain $R_HOME from R directly (encoding not support?)"))
        }
    } else {
        Err(Error::new(ErrorKind::Other, "Cannot find R home.\nNote: Either specific `$R_HOME` variable directly or leave R in `$PATH` could solve this issue.\n    The $R_HOME variable could be set automatically with `R CMD build` command if you are building an R package.\n    If you want to set it manually, its value could be the output of the command `R RHOME`"))
    }
}

// Get the path to the R library
fn get_r_library(r_home: &Path) -> PathBuf {
    let pkg_target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    match (cfg!(windows), pkg_target_arch.as_str()) {
        // For Windows
        (true, "x86_64") => Path::new(r_home).join("bin").join("x64"),
        (true, "x86") => Path::new(r_home).join("bin").join("i386"),
        (true, _) => panic!("Unknown architecture"),
        // For Unix-alike
        (false, _) => Path::new(r_home).join("lib"),
    }
}

// Get the path to the R include directory either from an envvar or by executing the actual R binary.
struct InstallationPaths {
    r_home: PathBuf,
    library: PathBuf,
}
fn probe_r_paths() -> io::Result<InstallationPaths> {
    let r_home = get_r_home()?;
    let library = get_r_library(&r_home);
    Ok(InstallationPaths { r_home, library })
}

fn main() {
    // println!("cargo:rustc-env=RUSTC_BOOTSTRAP=rmin");  // This should be done by make scripts.
    let r_paths = probe_r_paths();

    let r_paths = match r_paths {
        Ok(result) => result,
        Err(error) => {
            println!("Problem locating local R install: {:?}", error);
            exit(1);
        }
    };
    println!("cargo:rustc-env=R_HOME={}", r_paths.r_home.display());
    println!("cargo:r_home={}", r_paths.r_home.display()); // Becomes DEP_R_R_HOME for clients

    // std or core:
    // #[cfg(feature = "core")]
    // println!("cargo:rustc-cfg=have_no_std");

    println!("cargo::rustc-check-cfg=cfg(have_std)");
    #[cfg(not(feature = "core"))]
    println!("cargo:rustc-cfg=have_std");


    #[cfg(all(feature = "std", feature = "core"))]
    println!("cargo:warning=both `std` and `core` is enabled, enable `core` by default. May affect dependencies.");
    #[cfg(all(not(feature = "std"), not(feature = "core")))]
    println!("cargo:warning=neither `std` nor `core` is enabled, enable `std` by default.");

    // TODO: r_library might not exist in some types of installation that
    // doesn't provide libR, R's shared library; in such a situation, just skip
    // setting `rustc-link-search`. Probably this setting itself is not used at
    // all except when compiled for testing, but we are not sure at the moment.
    if let Ok(r_library) = r_paths.library.canonicalize() {
        println!("cargo:rustc-link-search={}", r_library.display());
    }
    println!("cargo:rerun-if-changed=build.rs");
}
