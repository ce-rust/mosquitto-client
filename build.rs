/// this file is mainly inspired by the great c library integration from https://github.com/eclipse/paho.mqtt.rust

fn main() {
    bundled::main();
}

const MOSQUITTO_GIT_URL: &str = "https://github.com/eclipse/mosquitto.git";
const MOSQUITTO_VERSION: &str = "2.0.4";

#[cfg(feature = "build_bindgen")]
mod bindings {
    use std::{env, fs};
    use std::path::{Path, PathBuf};
    use MOSQUITTO_VERSION;

    pub fn place_bindings(inc_dir: &Path) {
        let inc_search = format!("-I{}", inc_dir.display());

        // The bindgen::Builder is the main entry point
        // to bindgen, and lets you build up options for
        // the resulting bindings.
        let bindings = bindgen::Builder::default()
            // Older clang versions (~v3.6) improperly mangle the functions.
            // We shouldn't require mangling for straight C library. I think.
            .trust_clang_mangling(false)
            // The input header we would like to generate
            // bindings for.
            .header("wrapper.h").clang_arg(inc_search)
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let out_path = out_dir.join("bindings.rs");

        bindings
            .write_to_file(out_path.clone())
            .expect("Couldn't write bindings!");

        // Save a copy of the bindings file into the bindings/ dir
        // with version and target name, if it doesn't already exist

        let target = env::var("TARGET").unwrap();
        println!("debug:Target: {}", target);

        let bindings = format!("bindings/bindings_mosquitto_{}-{}.rs",
                               MOSQUITTO_VERSION, target);

        if !Path::new(&bindings).exists() {
            if let Err(err) = fs::copy(out_path, &bindings) {
                println!("debug:Error copying new binding file: {}", err);
            } else {
                println!("debug:Created new bindings file {}", bindings)
            }
        }
    }
}

#[cfg(feature = "bundled")]
mod bundled {
    use std::process::Command;
    use super::*;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::env;
    use std::fs;
    use std::str;

    extern crate anyhow;
    extern crate cmake;

    use self::anyhow::{Result, Error};
    #[cfg(target_os = "linux")]
    use self::anyhow::Context;

    struct LibInfos {
        lib_dir: PathBuf,
        lib_name: String,
        include_dir: PathBuf,
    }

    pub fn main() {
        println!("Running the bundled build");

        if let Err(e) = execute() {
            panic!("failed to build bundled library: {:?}", e)
        }
    }

    fn execute() -> Result<()> {
        checkout_lib()?;
        bundle_lib_and_link()
    }

    fn get_mosquitto_parent_dir() -> Result<PathBuf> {
        Ok(PathBuf::from(env::var("OUT_DIR")?))
    }

    fn get_mosquitto_dir() -> Result<PathBuf> {
        let p = get_mosquitto_parent_dir()?;
        Ok(p.join("mosquitto"))
    }

    #[cfg(target_os = "linux")]
    fn build_lib() -> Result<LibInfos> {
        let client_lib_dir = get_mosquitto_dir()?.join("lib");
        let cross_compiler = env::var("MOSQUITTO_CROSS_COMPILER").unwrap_or("".to_string());
        let cc_compiler = env::var("MOSQUITTO_CC").unwrap_or("gcc".to_string());
        Command::new("make")
            .env("CROSS_COMPILE", cross_compiler)
            .env("CC", cc_compiler)
            .current_dir(&client_lib_dir).args(&[
            "WITH_TLS=no",
            "WITH_CJSON=no",
            "all"
        ])
            .status().context("failed to make lib")?;
        Ok(LibInfos {
            lib_dir: client_lib_dir.clone(),
            lib_name: "libmosquitto.so.1".into(),
            include_dir: get_mosquitto_dir()?.join("include"),
        })
    }

    #[cfg(target_os = "macos")]
    fn build_lib() -> Result<LibInfos> {
        let mut cmk_cfg = cmake::Config::new(get_mosquitto_dir()?);
        let cmk = cmk_cfg.define("WITH_BUNDLED_DEPS", "on")
            .define("WITH_EC", "off")
            .define("WITH_TLS", "off")
            .define("WITH_TLS_PSK", "off")
            .define("WITH_APPS", "off")
            .define("WITH_PLUGINS", "off")
            .define("DOCUMENTATION", "off")
            .define("WITH_CJSON", "off")
            .define("CMAKE_VERBOSE_MAKEFILE", "on")
            .build();

        let lib_path = if cmk.join("lib").exists() {
            "lib"
        } else {
            panic!("Unknown library directory.")
        };

        Ok(LibInfos {
            lib_dir: cmk.join(lib_path),
            lib_name: "libmosquitto.1.dylib".into(),
            include_dir: cmk.join("include"),
        })
    }

    fn bundle_lib_and_link() -> Result<()> {
        let lib_info = build_lib()?;

        let lib = lib_info.lib_dir.join(Path::new(&lib_info.lib_name));
        println!("debug:Using mosquitto C library at: {}", lib.display());

        if !lib.exists() {
            println!("Error building mosquitto C library: '{}'", lib.display());
            process::exit(103);
        }

        // Get bundled bindings or regenerate
        bindings::place_bindings(&lib_info.include_dir);

        // we add the folder where all the libraries are built to the path search
        println!("cargo:rustc-link-search=native={}", lib_info.lib_dir.display());
        println!("cargo:rustc-link-lib={}", "mosquitto");
        Ok(())
    }

    fn checkout_lib() -> Result<()> {
        let git_parent_path = get_mosquitto_parent_dir()?;
        let git_path = get_mosquitto_dir()?;
        println!("checkout_lib to {} in {}", git_path.to_str().unwrap(), git_parent_path.to_str().unwrap());
        if git_path.is_dir() {
            fs::remove_dir_all(&git_path)?;
        } else if !git_parent_path.is_dir() {
            fs::create_dir_all(&git_parent_path)?;
        }

        if !git_parent_path.is_dir() {
            return Err(Error::msg("was not able to create directory"));
        }

        let args = vec![
            "clone".to_string(),
            "--depth=1".to_string(),
            env::var("MOSQUITTO_GIT_URL").unwrap_or(MOSQUITTO_GIT_URL.to_string()),
            git_path.to_str().unwrap().to_string(),
        ];

        if let Err(e) = Command::new("git").current_dir(&git_parent_path).args(&args).status() {
            panic!("failed to clone the git repo: {:?}", e);
        }

        let hash = env::var("MOSQUITTO_GIT_HASH");
        if let Ok(hash) = hash.as_ref() {
            if let Err(e) = Command::new("git").current_dir(&git_path).args(&["fetch", "--depth", "1", "origin", hash.as_str()]).status() {
                panic!("failed to fetch the git hash: {:?}", e);
            }
            if let Err(e) = Command::new("git").current_dir(&git_path).args(&["checkout", hash.as_str()]).status() {
                panic!("failed to checkout the git hash: {:?}", e);
            }
        }

        let output = Command::new("git").current_dir(&git_path).args(&["rev-parse", "HEAD"]).output()?;
        let output = str::from_utf8(&output.stdout)?;
        if let Ok(hash) = hash.as_ref() {
            if output.ne(format!("{}\n", hash).as_str()) {
                panic!("was not able to get correct hash: found {}, expected {}", output, hash);
            } else {
                println!("debug:Hash: {}", output);
                Ok(())
            }
        } else {
            println!("debug:Hash: {}", output);
            Ok(())
        }
    }
}

#[cfg(not(feature = "bundled"))]
mod bundled {
    pub fn main() {}
}
