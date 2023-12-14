extern crate cc;

use std::env;

fn main() {
    // Check whether we can use 64-bit compilation
    let use_64bit_compilation = if env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap() == "64" {
        let check = cc::Build::new()
            .file("depend/check_uint128_t.c")
            .cargo_metadata(false)
            .try_compile("check_uint128_t")
            .is_ok();
        if !check {
            println!("cargo:warning=Compiling in 32-bit mode on a 64-bit architecture due to lack of uint128_t support.");
        }
        check
    } else {
        false
    };
    let target = env::var("TARGET").expect("TARGET was not set");
    let is_big_endian = env::var("CARGO_CFG_TARGET_ENDIAN").expect("No endian is set") == "big";
    // **Secp256k1**
    if !cfg!(feature = "external-secp") {
        let mut base_config = cc::Build::new();
        base_config
            .include("depend/bitcoin/src")
            .include("depend/bitcoin/src/secp256k1/include")
            .define("__STDC_FORMAT_MACROS", None);

        base_config
            .include("depend/secp256k1/")
            .include("depend/secp256k1/include")
            .include("depend/secp256k1/src")
            .flag_if_supported("-Wno-unused-function") // some ecmult stuff is defined but not used upstream
            .flag_if_supported("-Wno-unused-parameter") // patching out printf causes this warning
            .define("SECP256K1_API", Some(""))
            .define("ENABLE_MODULE_ECDH", Some("1"))
            .define("ENABLE_MODULE_SCHNORRSIG", Some("1"))
            .define("ENABLE_MODULE_EXTRAKEYS", Some("1"))
            .define("ENABLE_MODULE_ELLSWIFT", Some("1"))
            // upstream sometimes introduces calls to printf, which we cannot compile
            // with WASM due to its lack of libc. printf is never necessary and we can
            // just #define it away.
            .define("printf(...)", Some(""));

        base_config
            .include("depend/bitcoin/src/secp256k1")
            .flag_if_supported("-Wno-unused-function") // some ecmult stuff is defined but not used upstream
            .define("SECP256K1_BUILD", "1")
            // Bitcoin core defines libsecp to *not* use libgmp.
            .define("USE_NUM_NONE", "1")
            .define("USE_FIELD_INV_BUILTIN", "1")
            .define("USE_SCALAR_INV_BUILTIN", "1")
            // Technically libconsensus doesn't require the recovery feautre, but `pubkey.cpp` does.
            .define("ENABLE_MODULE_RECOVERY", "1")
            // The actual libsecp256k1 C code.
            .file("depend/bitcoin/src/secp256k1/contrib/lax_der_parsing.c")
            .file("depend/bitcoin/src/secp256k1/src/precomputed_ecmult_gen.c")
            .file("depend/bitcoin/src/secp256k1/src/precomputed_ecmult.c")
            .file("depend/bitcoin/src/secp256k1/src/secp256k1.c");

        if is_big_endian {
            base_config.define("WORDS_BIGENDIAN", "1");
        }

        if use_64bit_compilation {
            base_config
                .define("USE_FIELD_5X52", "1")
                .define("USE_SCALAR_4X64", "1")
                .define("HAVE___INT128", "1");
        } else {
            base_config.define("USE_FIELD_10X26", "1").define("USE_SCALAR_8X32", "1");
        }

        if cfg!(feature = "lowmemory") {
            base_config.define("ECMULT_WINDOW_SIZE", Some("4")); // A low-enough value to consume negligible memory
            base_config.define("ECMULT_GEN_PREC_BITS", Some("2"));
        } else {
            base_config.define("ECMULT_GEN_PREC_BITS", Some("4"));
            base_config.define("ECMULT_WINDOW_SIZE", Some("15")); // This is the default in the configure file (`auto`)
        }
        base_config.define("USE_EXTERNAL_DEFAULT_CALLBACKS", Some("1"));
        #[cfg(feature = "recovery")]
        base_config.define("ENABLE_MODULE_RECOVERY", Some("1"));

        // WASM headers and size/align defines.
        if env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "wasm32" {
            base_config.include("wasm/wasm-sysroot").file("wasm/wasm.c");
        }

        base_config.compile("secp256k1");
    }

    let mut base_config = cc::Build::new();
    base_config
        .include("depend/bitcoin/src")
        .include("depend/bitcoin/src/secp256k1/include")
        .define("__STDC_FORMAT_MACROS", None);
    base_config.object("secp256k1").cpp(true);

    let tool = base_config.get_compiler();
    if tool.is_like_msvc() {
        base_config.std("c++14").flag("/wd4100");
    } else if tool.is_like_clang() || tool.is_like_gnu() {
        base_config.std("c++11").flag("-Wno-unused-parameter");
    }

    if target.contains("windows") {
        base_config.define("WIN32", "1");
    }

    if target.contains("emscripten") {
        base_config.compiler("emcc").flag("--no-entry").define("ERROR_ON_UNDEFINED_SYMBOLS", "0");
    } else if target.contains("wasm") {
        if target.contains("wasi") {
            base_config.include("/usr/include/wasm32-wasi");
        }

        base_config
            .include("/usr/include")
            .include("/usr/include/c++/11")
            .include("/usr/include/x86_64-linux-gnu")
            .include("/usr/include/x86_64-linux-gnu/c++/11");
    }

    base_config
        .file("depend/bitcoin/src/util/strencodings.cpp")
        .file("depend/bitcoin/src/uint256.cpp")
        .file("depend/bitcoin/src/pubkey.cpp")
        .file("depend/bitcoin/src/hash.cpp")
        .file("depend/bitcoin/src/primitives/transaction.cpp")
        .file("depend/bitcoin/src/crypto/ripemd160.cpp")
        .file("depend/bitcoin/src/crypto/sha1.cpp")
        .file("depend/bitcoin/src/crypto/sha256.cpp")
        .file("depend/bitcoin/src/crypto/sha512.cpp")
        .file("depend/bitcoin/src/crypto/hmac_sha512.cpp")
        .file("depend/bitcoin/src/script/bitcoinconsensus.cpp")
        .file("depend/bitcoin/src/script/interpreter.cpp")
        .file("depend/bitcoin/src/script/script.cpp")
        .file("depend/bitcoin/src/script/script_error.cpp")
        .compile("bitcoinconsensus");
}
