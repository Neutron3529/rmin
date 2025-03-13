#!/bin/bash
EXTRA_CARGO_CONFIG=${EXTRA_CARGO_CONFIG:-${1:-.cargo.config.toml}}

if [ x"$1" == x"clean" ] ; then
    rm -rf R/aaa.rmin.Rust.Functions.R src/{Makevars{,.win},symbols.rds,*.so} Cargo.lock vendor .cargo/config.toml .cargo man
    exit
elif [ x"$1" == x"help" ] ; then
    echo "Usage:"
    echo "  ./pack         # pack the package automatically"
    echo "  ./pack noclean # pack the package automatically, without clean files after a success build."
    echo "  ./pack clean   # clean the packed files."
fi

makefile(){
  echo STATLIB = '$(LIBDIR)/'$2 > $1
  cat >>$1<<\EOF
TARGET_DIR = ./../target
LIBDIR = $(TARGET_DIR)/release
R_SCRIPT_DIR = ./../R/

all: rust_clean

$(SHLIB): $(STATLIB)

CARGOTMP = $(CURDIR)/.cargo

$(STATLIB):
	# In some environments, ~/.cargo/bin might not be included in PATH, so we need
	# to set it here to ensure cargo can be invoked. It is appended to PATH and
	# therefore is only used if cargo is absent from the user's PATH.
	if [ "$(NOT_CRAN)" != "true" ]; then \
		export CARGO_HOME=$(CARGOTMP); \
	fi && \
		export PATH="$(PATH):$(HOME)/.cargo/bin" && \
		cargo build --lib --release --manifest-path=$(TARGET_DIR)/../Cargo.toml --target-dir $(TARGET_DIR) && \
		cp $(STATLIB) $(SHLIB) && \
	if [ "$(NOT_CRAN)" != "true" ]; then \
		rm -Rf $(CARGOTMP) && \
		rm -rf $(TARGET_DIR) ; \
	fi

rust_clean:
	rm -Rf $(SHLIB) $(STATLIB) $(OBJECTS)

clean:
	rm -Rf $(SHLIB) $(STATLIB) $(OBJECTS) $(TARGET_DIR)
EOF
}

copy_config(){
    if [ -e "$EXTRA_CARGO_CONFIG" ] ; then
        cp $EXTRA_CARGO_CONFIG .cargo/config.toml
    fi
}

if [ -d .cargo ] ; then
    echo "pack.sh: .cargo should be generate automatically, please use environment EXTRA_CARGO_CONFIG to tell pack.sh what the extra defines should be"
    echo "Example:"
    echo "  EXTRA_CARGO_CONFIG=../my_config ./pack.sh"
    echo
    exit
fi
mkdir .cargo

DIR="$(dirname $0)"
echo cd $DIR
cd $DIR
DIR=$(pwd)
crate_name=$(sed -n '/name = /s/^name = "\(.*\)"$/\1/gp' Cargo.toml)
makefile src/Makevars "lib${crate_name}.so"
makefile src/Makevars.win "${crate_name}.dll"

if [ -d "vendor" ] ; then
    echo has vendor folder, skipping vendor.
    copy_config
else
    rm -f .cargo/config.toml
    copy_config
    cargo vendor --respect-source-config # allow using mirrors for vendoring, thus custom crates could be also vendored.
fi
cat >>.cargo/config.toml<<EOF
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

cargo check # generate aaa.rmin.Rust.Functions.R
R -s -e "cat('try generating documents\n');try(roxygen2::roxygenize('.'));cat('documents generating done.\n')"
if R CMD check $DIR ; then
    echo pack.sh: check passed.
    rm -rf $DIR.Rcheck
    if R CMD build $DIR ; then
        if [ x"$1" == x"noclean" ] ; then
            echo build done, execute \`./pack.sh clean\` to remove extra files.
        else
            echo build done, remove extra files.
            rm -rf R/aaa.rmin.Rust.Functions.R src/{Makevars{,.win},symbols.rds,*.so} Cargo.lock vendor .cargo/config.toml .cargo man
        fi
            exit
    else
        echo pack.sh: R CMD build failed.
    fi
else
    echo pack.sh: R CMD check failed.
fi

cd $OLDPWD
rm -r .cargo
