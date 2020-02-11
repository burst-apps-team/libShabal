use cfg_if::cfg_if;

use std::env;
use std::path::PathBuf;

fn main() {
    let mut shared_config = cc::Build::new();

    #[cfg(target_env = "msvc")]
        shared_config
        .flag("/O2")
        .flag("/Oi")
        .flag("/Ot")
        .flag("/Oy")
        .flag("/GT")
        .flag("/GL");

    #[cfg(not(target_env = "msvc"))]
        shared_config.flag("-std=c99");

    #[cfg(not(target_env = "msvc"))]
        shared_config.flag("-mtune=native");

    let mut config = shared_config.clone();

    config
        .file("src/pocc/c/sph_shabal.c")
        .file("src/pocc/c/shabal.c")
        .file("src/pocc/c/common.c")
        .compile("shabal");

    config = shared_config.clone();
    config
        .file("src/c/curve25519.c")
        .compile("curve25519");

    generate_bindings();

    cfg_if! {
         if #[cfg(feature = "neon")] {
             fn build(shared_config: cc::Build) {
                let mut config = shared_config.clone();

                #[cfg(all(not(target_env = "msvc"), not(target_arch = "aarch64")))]
                config.flag("-mfpu=neon");

                config
                    .file("src/pocc/c/mshabal_128_neon.c")
                    .file("src/pocc/c/shabal_neon.c")
                    .compile("shabal_neon");
             }
         }
    }

    cfg_if! {
         if #[cfg(feature = "simd")] {
             fn build(shared_config: cc::Build) {
                let mut config = shared_config.clone();

                #[cfg(not(target_env = "msvc"))]
                config.flag("-msse2");

                config
                    .file("src/pocc/c/mshabal_128_sse2.c")
                    .file("src/pocc/c/shabal_sse2.c")
                    .file("src/pocc/c/noncegen_128_sse2.c")
                    .compile("shabal_sse2");

                let mut config = shared_config.clone();

                #[cfg(target_env = "msvc")]
                config.flag("/arch:AVX");

                #[cfg(not(target_env = "msvc"))]
                config.flag("-mavx");

                config
                    .file("src/pocc/c/mshabal_128_avx.c")
                    .file("src/pocc/c/shabal_avx.c")
                    .file("src/pocc/c/noncegen_128_avx.c")
                    .compile("shabal_avx");

                let mut config = shared_config.clone();

                #[cfg(target_env = "msvc")]
                config.flag("/arch:AVX2");

                #[cfg(not(target_env = "msvc"))]
                config.flag("-mavx2");

                config
                    .file("src/pocc/c/mshabal_256_avx2.c")
                    .file("src/pocc/c/shabal_avx2.c")
                    .file("src/pocc/c/noncegen_256_avx2.c")
                    .compile("shabal_avx2");

                let mut config = shared_config.clone();

                #[cfg(target_env = "msvc")]
                config.flag("/arch:AVX512F");

                #[cfg(not(target_env = "msvc"))]
                config.flag("-mavx512f");

                config
                    .file("src/pocc/c/mshabal_512_avx512f.c")
                    .file("src/pocc/c/shabal_avx512f.c")
                    .file("src/pocc/c/noncegen_512_avx512f.c")
                    .compile("shabal_avx512f");
            }
        }
    }

    #[cfg(any(feature = "simd", feature = "neon"))]
        build(shared_config);
}

fn generate_bindings() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = target_dir()
        .join(format!("{}.h", package_name))
        .display()
        .to_string();

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(output_file);
}

fn target_dir() -> PathBuf {
    if let Ok(target) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(target)
    } else {
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("target")
    }
}
