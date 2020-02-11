use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub enum SimdExtension {
    AVX512f,
    AVX2,
    AVX,
    SSE2,
    NEON,
    NONE,
}

pub static SUPPORTED_SIMD_EXTENSION: Lazy<SimdExtension> = Lazy::new(|| {
    #[cfg(feature = "simd")] {
        if is_x86_feature_detected!("avx512f") {
            return SimdExtension::AVX512f;
        } else if is_x86_feature_detected!("avx2") {
            return SimdExtension::AVX2;
        } else if is_x86_feature_detected!("avx") {
            return SimdExtension::AVX;
        } else if is_x86_feature_detected!("sse2") {
            return SimdExtension::SSE2;
        }
    }
    #[cfg(feature = "neon")] unsafe {
        #[cfg(target_arch = "arm")]
            let neon = is_arm_feature_detected!("neon");
        #[cfg(target_arch = "aarch64")]
            let neon = true;

        if neon {
            return SimdExtension::NEON;
        }
    }
    return SimdExtension::NONE;
});
