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
    pub projector: Option<ArtifactSpec>,
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
            Self::Android => ANDROID_GEMMA4_E2B_LITERTLM,
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
    projector: Some(ArtifactSpec {
        repository: "unsloth/Qwen3.5-9B-GGUF",
        revision: "3885219b6810b007914f3a7950a8d1b469d598a5",
        file_name: "mmproj-F16.gguf",
        url: "https://huggingface.co/unsloth/Qwen3.5-9B-GGUF/resolve/3885219b6810b007914f3a7950a8d1b469d598a5/mmproj-F16.gguf",
        byte_len: 918_166_080,
        sha256: "f70dc3509053962b0d0d3ee8a7eacebf5d60aa560cad78254ae8698516ae029f",
    }),
};

pub const ANDROID_QWEN3_4B: ModelArtifactSet = ModelArtifactSet {
    model_id: "qwen3-4b-instruct-2507-q5-k-m",
    model: ArtifactSpec {
        repository: "bartowski/Qwen_Qwen3-4B-Instruct-2507-GGUF",
        revision: "ae44f08e1392f39c0e474af10c3ff8355c8b6688",
        file_name: "Qwen_Qwen3-4B-Instruct-2507-Q5_K_M.gguf",
        url: "https://huggingface.co/bartowski/Qwen_Qwen3-4B-Instruct-2507-GGUF/resolve/ae44f08e1392f39c0e474af10c3ff8355c8b6688/Qwen_Qwen3-4B-Instruct-2507-Q5_K_M.gguf",
        byte_len: 2_889_513_696,
        sha256: "66713ce35a58a82fe87642d4ec13425bf9b9a46800fff5c49a665ef5701439dc",
    },
    projector: None,
};

pub const ANDROID_GEMMA4_E2B_LITERTLM: ModelArtifactSet = ModelArtifactSet {
    model_id: "gemma-4-e2b-it-litertlm",
    model: ArtifactSpec {
        repository: "litert-community/gemma-4-E2B-it-litert-lm",
        revision: "9262660a1676eed6d0c477ab1a86344430854664",
        file_name: "gemma-4-E2B-it.litertlm",
        url: "https://huggingface.co/litert-community/gemma-4-E2B-it-litert-lm/resolve/9262660a1676eed6d0c477ab1a86344430854664/gemma-4-E2B-it.litertlm",
        byte_len: 2_588_147_712,
        sha256: "181938105e0eefd105961417e8da75903eacda102c4fce9ce90f50b97139a63c",
    },
    projector: None,
};

pub const ANDROID_GEMMA4_E2B_SM8750_LITERTLM: ModelArtifactSet = ModelArtifactSet {
    model_id: "gemma-4-e2b-it-litertlm-sm8750",
    model: ArtifactSpec {
        repository: "litert-community/gemma-4-E2B-it-litert-lm",
        revision: "9262660a1676eed6d0c477ab1a86344430854664",
        file_name: "gemma-4-E2B-it_qualcomm_sm8750.litertlm",
        url: "https://huggingface.co/litert-community/gemma-4-E2B-it-litert-lm/resolve/9262660a1676eed6d0c477ab1a86344430854664/gemma-4-E2B-it_qualcomm_sm8750.litertlm",
        byte_len: 3_016_294_400,
        sha256: "41dd675fbe735b6029012b5576a5716bac614fd8156de0128db4c9dff3cebd4e",
    },
    projector: None,
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
        ANDROID_GEMMA4_E2B_LITERTLM_ID => Some(ANDROID_GEMMA4_E2B_LITERTLM),
        ANDROID_GEMMA4_E2B_SM8750_LITERTLM_ID => Some(ANDROID_GEMMA4_E2B_SM8750_LITERTLM),
        ANDROID_QWEN3_4B_ID => Some(ANDROID_QWEN3_4B),
        _ => None,
    }
}

pub const IPHONE_QWEN35_9B_ID: &str = IPHONE_QWEN35_9B.model_id;
pub const ANDROID_GEMMA4_E2B_LITERTLM_ID: &str = ANDROID_GEMMA4_E2B_LITERTLM.model_id;
pub const ANDROID_GEMMA4_E2B_SM8750_LITERTLM_ID: &str = ANDROID_GEMMA4_E2B_SM8750_LITERTLM.model_id;
pub const ANDROID_QWEN3_4B_ID: &str = ANDROID_QWEN3_4B.model_id;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifacts_are_revision_pinned_and_sized() {
        for set in [
            IPHONE_QWEN35_9B,
            ANDROID_GEMMA4_E2B_LITERTLM,
            ANDROID_GEMMA4_E2B_SM8750_LITERTLM,
            ANDROID_QWEN3_4B,
        ] {
            for artifact in std::iter::once(set.model).chain(set.projector) {
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
            ANDROID_GEMMA4_E2B_LITERTLM_ID
        );
        assert_ne!(
            MobilePlatform::Ios.default_model().model_id,
            MobilePlatform::Android.default_model().model_id
        );
        assert_eq!(
            artifact_set_for_id(IPHONE_QWEN35_9B_ID),
            Some(IPHONE_QWEN35_9B)
        );
        assert_eq!(
            artifact_set_for_id(ANDROID_GEMMA4_E2B_LITERTLM_ID),
            Some(ANDROID_GEMMA4_E2B_LITERTLM)
        );
    }
}
