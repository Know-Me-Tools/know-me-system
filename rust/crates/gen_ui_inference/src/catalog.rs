//! Immutable mobile model artifact catalog.
//!
//! The digests are the SHA-256 values exposed by Hugging Face's immutable LFS
//! metadata (`X-Linked-Etag`).  URLs are revision-pinned so a mutable `main`
//! branch cannot silently replace a model after release.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactSpec {
    pub repository: &'static str,
    pub revision: &'static str,
    pub file_name: &'static str,
    pub url: &'static str,
    pub byte_len: u64,
    pub sha256: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelArtifactSet {
    pub model_id: &'static str,
    pub model: ArtifactSpec,
    pub projector: ArtifactSpec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobilePlatform {
    Ios,
    Android,
}

impl MobilePlatform {
    pub const fn default_model(self) -> ModelArtifactSet {
        match self {
            Self::Ios => IPHONE_QWEN35_9B,
            Self::Android => ANDROID_QWEN35_4B,
        }
    }
}

pub const IPHONE_QWEN35_9B: ModelArtifactSet = ModelArtifactSet {
    model_id: "qwen3.5-9b-q4-k-m",
    model: ArtifactSpec {
        repository: "unsloth/Qwen3.5-9B-GGUF",
        revision: "3885219b6810b007914f3a7950a8d1b469d598a5",
        file_name: "Qwen3.5-9B-Q4_K_M.gguf",
        url: "https://huggingface.co/unsloth/Qwen3.5-9B-GGUF/resolve/3885219b6810b007914f3a7950a8d1b469d598a5/Qwen3.5-9B-Q4_K_M.gguf",
        byte_len: 5_680_522_464,
        sha256: "03b74727a860a56338e042c4420bb3f04b2fec5734175f4cb9fa853daf52b7e8",
    },
    projector: ArtifactSpec {
        repository: "unsloth/Qwen3.5-9B-GGUF",
        revision: "3885219b6810b007914f3a7950a8d1b469d598a5",
        file_name: "mmproj-F16.gguf",
        url: "https://huggingface.co/unsloth/Qwen3.5-9B-GGUF/resolve/3885219b6810b007914f3a7950a8d1b469d598a5/mmproj-F16.gguf",
        byte_len: 918_166_080,
        sha256: "f70dc3509053962b0d0d3ee8a7eacebf5d60aa560cad78254ae8698516ae029f",
    },
};

pub const ANDROID_QWEN35_4B: ModelArtifactSet = ModelArtifactSet {
    model_id: "qwen3.5-4b-q4-k-m",
    model: ArtifactSpec {
        repository: "unsloth/Qwen3.5-4B-GGUF",
        revision: "e87f176479d0855a907a41277aca2f8ee7a09523",
        file_name: "Qwen3.5-4B-Q4_K_M.gguf",
        url: "https://huggingface.co/unsloth/Qwen3.5-4B-GGUF/resolve/e87f176479d0855a907a41277aca2f8ee7a09523/Qwen3.5-4B-Q4_K_M.gguf",
        byte_len: 2_740_937_888,
        sha256: "00fe7986ff5f6b463e62455821146049db6f9313603938a70800d1fb69ef11a4",
    },
    projector: ArtifactSpec {
        repository: "unsloth/Qwen3.5-4B-GGUF",
        revision: "e87f176479d0855a907a41277aca2f8ee7a09523",
        file_name: "mmproj-F16.gguf",
        url: "https://huggingface.co/unsloth/Qwen3.5-4B-GGUF/resolve/e87f176479d0855a907a41277aca2f8ee7a09523/mmproj-F16.gguf",
        byte_len: 672_423_616,
        sha256: "cd88edcf8d031894960bb0c9c5b9b7e1fea6ebee02b9f7ce925a00d12891f864",
    },
};

pub fn current_mobile_platform() -> Option<MobilePlatform> {
    #[cfg(target_os = "ios")]
    {
        return Some(MobilePlatform::Ios);
    }
    #[cfg(target_os = "android")]
    {
        return Some(MobilePlatform::Android);
    }
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    {
        None
    }
}

pub fn artifact_set_for_id(model_id: &str) -> Option<ModelArtifactSet> {
    match model_id {
        IPHONE_QWEN35_9B_ID => Some(IPHONE_QWEN35_9B),
        ANDROID_QWEN35_4B_ID => Some(ANDROID_QWEN35_4B),
        _ => None,
    }
}

pub const IPHONE_QWEN35_9B_ID: &str = IPHONE_QWEN35_9B.model_id;
pub const ANDROID_QWEN35_4B_ID: &str = ANDROID_QWEN35_4B.model_id;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifacts_are_revision_pinned_and_sized() {
        for set in [IPHONE_QWEN35_9B, ANDROID_QWEN35_4B] {
            for artifact in [set.model, set.projector] {
                assert!(!artifact.revision.is_empty());
                assert!(artifact.url.contains(artifact.revision));
                assert!(artifact.byte_len > 0);
                assert_eq!(artifact.sha256.len(), 64);
            }
        }
    }

    #[test]
    fn platform_defaults_are_typed_and_distinct() {
        assert_eq!(
            MobilePlatform::Ios.default_model().model_id,
            IPHONE_QWEN35_9B_ID
        );
        assert_eq!(
            MobilePlatform::Android.default_model().model_id,
            ANDROID_QWEN35_4B_ID
        );
        assert_ne!(
            MobilePlatform::Ios.default_model().model_id,
            MobilePlatform::Android.default_model().model_id
        );
        assert_eq!(
            artifact_set_for_id(IPHONE_QWEN35_9B_ID),
            Some(IPHONE_QWEN35_9B)
        );
    }
}
