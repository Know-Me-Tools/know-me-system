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

    /// The platform default by id alone. iOS's default is an MLX multi-file
    /// set that `ModelArtifactSet` cannot represent, so callers that only need
    /// the id (boot's default-model selection) use this instead of
    /// `default_model()`.
    pub const fn default_model_id(self) -> &'static str {
        match self {
            Self::Ios => IOS_QWEN35_4B_MLX.model_id,
            Self::Android => ANDROID_GEMMA4_E2B_LITERTLM.model_id,
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

/// A multi-file model artifact set for MLX-format models.
///
/// MLX models are Hugging Face repos (config + tokenizer + safetensors
/// shards), not single files, so pinning means pinning every file. Rust owns
/// the download of each file — resumable, revision-pinned, SHA-256-verified —
/// into one model directory that the platform bridge receives whole. `files`
/// is in download order: small config files first so a bad revision fails
/// before any multi-gigabyte shard transfer starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MlxArtifactSet {
    pub model_id: &'static str,
    pub repository: &'static str,
    pub revision: &'static str,
    pub files: &'static [ArtifactSpec],
    pub total_bytes: u64,
    /// Memory-preflight input: weights plus KV cache at the trusted context
    /// plus Metal buffer overhead. MLX's unified-memory residency runs well
    /// above the artifact bytes, unlike llama.cpp's mmap profile. Estimated
    /// until tuned against on-device jetsam evidence.
    pub estimated_peak_bytes: u64,
}

macro_rules! mlx_file {
    ($repo:literal, $rev:literal, $name:literal, $bytes:literal, $sha:literal) => {
        ArtifactSpec {
            repository: $repo,
            revision: $rev,
            file_name: $name,
            url: concat!(
                "https://huggingface.co/",
                $repo,
                "/resolve/",
                $rev,
                "/",
                $name
            ),
            byte_len: $bytes,
            sha256: $sha,
        }
    };
}

/// Qwen3.5-4B 4-bit MLX — the iOS default. Reasoning-tuned, fits the 8 GB
/// device tier. Text lane today: the base model is natively multimodal, but
/// MLXVLM's Swift registry currently ships Qwen3.5 text-only (`qwen3_5`, no
/// vision processor). The vision-capable upgrade is a catalog swap once the
/// processor lands upstream — the preprocessor/processor configs are already
/// pinned here so that swap needs no new files.
pub const IOS_QWEN35_4B_MLX: MlxArtifactSet = MlxArtifactSet {
    model_id: "qwen3.5-4b-mlx-4bit",
    repository: "mlx-community/Qwen3.5-4B-MLX-4bit",
    revision: "32f3e8ecf65426fc3306969496342d504bfa13f3",
    files: &[
        mlx_file!(
            "mlx-community/Qwen3.5-4B-MLX-4bit",
            "32f3e8ecf65426fc3306969496342d504bfa13f3",
            "config.json",
            3_366,
            "f3efc81b2ea8d96a45301037d3ccccbcccdef44a961845c87f286aaddbc6eaaa"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-4B-MLX-4bit",
            "32f3e8ecf65426fc3306969496342d504bfa13f3",
            "tokenizer_config.json",
            1_139,
            "e98f1901ac6f0adff67b1d540bfa0c36ac1a0cf59eb72ed78146ef89aafa1182"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-4B-MLX-4bit",
            "32f3e8ecf65426fc3306969496342d504bfa13f3",
            "chat_template.jinja",
            7_756,
            "a4aee8afcf2e0711942cf848899be66016f8d14a889ff9ede07bca099c28f715"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-4B-MLX-4bit",
            "32f3e8ecf65426fc3306969496342d504bfa13f3",
            "preprocessor_config.json",
            390,
            "27225450ac9c6529872ee1924fcb0962ff5634834f817040f444118116f4e516"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-4B-MLX-4bit",
            "32f3e8ecf65426fc3306969496342d504bfa13f3",
            "processor_config.json",
            1_300,
            "14932921ca485d458a04dafd8069fbb0a4505622a48208d19ed247115801385b"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-4B-MLX-4bit",
            "32f3e8ecf65426fc3306969496342d504bfa13f3",
            "video_preprocessor_config.json",
            385,
            "7768af27c1fafa9cc9011c1dc20067e03f8915e03b63504550e11d5066986d13"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-4B-MLX-4bit",
            "32f3e8ecf65426fc3306969496342d504bfa13f3",
            "model.safetensors.index.json",
            101_944,
            "52e534c41f7b97708329c85f762e5882bf48bd5955a422c6ae74eba321e6048a"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-4B-MLX-4bit",
            "32f3e8ecf65426fc3306969496342d504bfa13f3",
            "vocab.json",
            6_722_759,
            "ce99b4cb2983d118806ce0a8b777a35b093e2000a503ebde25853284c9dfa003"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-4B-MLX-4bit",
            "32f3e8ecf65426fc3306969496342d504bfa13f3",
            "tokenizer.json",
            19_989_343,
            "87a7830d63fcf43bf241c3c5242e96e62dd3fdc29224ca26fed8ea333db72de4"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-4B-MLX-4bit",
            "32f3e8ecf65426fc3306969496342d504bfa13f3",
            "model.safetensors",
            3_034_300_695,
            "5fb9acd0246866381cf8c5c354c6db1019f6498eec4ccb4f5edcc71ffeacb2db"
        ),
    ],
    total_bytes: 3_061_129_077,
    estimated_peak_bytes: 4_400_000_000,
};

/// Qwen3.5-9B 4-bit MLX — the 12 GB-device opt-in tier (the Qwen3.5 line has
/// no 8B; 9B is the mid-size). Never a default: the memory preflight admits it
/// only where the jetsam budget clears `estimated_peak_bytes`, which today is
/// marginal even on 12 GB — expected to be confirmed or rejected by on-device
/// tuning rather than assumed.
pub const IOS_QWEN35_9B_MLX: MlxArtifactSet = MlxArtifactSet {
    model_id: "qwen3.5-9b-mlx-4bit",
    repository: "mlx-community/Qwen3.5-9B-MLX-4bit",
    revision: "938d8919941c6e7efd3c7150eff7fe9d12afa631",
    files: &[
        mlx_file!(
            "mlx-community/Qwen3.5-9B-MLX-4bit",
            "938d8919941c6e7efd3c7150eff7fe9d12afa631",
            "config.json",
            3_331,
            "a96942cb6a8a1d3f1d17514d81a1925d04362a6a3233b389d13012211baaa9f8"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-9B-MLX-4bit",
            "938d8919941c6e7efd3c7150eff7fe9d12afa631",
            "tokenizer_config.json",
            1_139,
            "e98f1901ac6f0adff67b1d540bfa0c36ac1a0cf59eb72ed78146ef89aafa1182"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-9B-MLX-4bit",
            "938d8919941c6e7efd3c7150eff7fe9d12afa631",
            "chat_template.jinja",
            7_756,
            "a4aee8afcf2e0711942cf848899be66016f8d14a889ff9ede07bca099c28f715"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-9B-MLX-4bit",
            "938d8919941c6e7efd3c7150eff7fe9d12afa631",
            "preprocessor_config.json",
            390,
            "27225450ac9c6529872ee1924fcb0962ff5634834f817040f444118116f4e516"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-9B-MLX-4bit",
            "938d8919941c6e7efd3c7150eff7fe9d12afa631",
            "processor_config.json",
            1_300,
            "14932921ca485d458a04dafd8069fbb0a4505622a48208d19ed247115801385b"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-9B-MLX-4bit",
            "938d8919941c6e7efd3c7150eff7fe9d12afa631",
            "video_preprocessor_config.json",
            385,
            "7768af27c1fafa9cc9011c1dc20067e03f8915e03b63504550e11d5066986d13"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-9B-MLX-4bit",
            "938d8919941c6e7efd3c7150eff7fe9d12afa631",
            "model.safetensors.index.json",
            123_592,
            "dd023913fb87cfdae27fb11dcf695117c925833796ccac3c64117d6652d8ff1e"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-9B-MLX-4bit",
            "938d8919941c6e7efd3c7150eff7fe9d12afa631",
            "vocab.json",
            6_722_759,
            "ce99b4cb2983d118806ce0a8b777a35b093e2000a503ebde25853284c9dfa003"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-9B-MLX-4bit",
            "938d8919941c6e7efd3c7150eff7fe9d12afa631",
            "tokenizer.json",
            19_989_343,
            "87a7830d63fcf43bf241c3c5242e96e62dd3fdc29224ca26fed8ea333db72de4"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-9B-MLX-4bit",
            "938d8919941c6e7efd3c7150eff7fe9d12afa631",
            "model-00001-of-00002.safetensors",
            5_349_771_222,
            "a68b87558c6ef43f74c2bd63ce7e9092ceddc3101f3def0030774bae5f42aadd"
        ),
        mlx_file!(
            "mlx-community/Qwen3.5-9B-MLX-4bit",
            "938d8919941c6e7efd3c7150eff7fe9d12afa631",
            "model-00002-of-00002.safetensors",
            600_449_850,
            "b0a770bf8469c7f3f18756a0e0283f1c1174344a83e059a4e483f6af4907352d"
        ),
    ],
    total_bytes: 5_977_071_067,
    estimated_peak_bytes: 7_500_000_000,
};

pub const IOS_QWEN35_4B_MLX_ID: &str = IOS_QWEN35_4B_MLX.model_id;
pub const IOS_QWEN35_9B_MLX_ID: &str = IOS_QWEN35_9B_MLX.model_id;

pub fn mlx_artifact_set_for_id(model_id: &str) -> Option<&'static MlxArtifactSet> {
    match model_id {
        IOS_QWEN35_4B_MLX_ID => Some(&IOS_QWEN35_4B_MLX),
        IOS_QWEN35_9B_MLX_ID => Some(&IOS_QWEN35_9B_MLX),
        _ => None,
    }
}

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

    #[test]
    fn mlx_file_sets_are_revision_pinned_sized_and_summed() {
        for set in [&IOS_QWEN35_4B_MLX, &IOS_QWEN35_9B_MLX] {
            assert!(!set.revision.is_empty());
            assert!(!set.files.is_empty());
            let mut sum = 0u64;
            for file in set.files {
                assert_eq!(file.repository, set.repository);
                assert_eq!(file.revision, set.revision);
                assert!(file.url.contains(set.revision));
                assert!(file.url.ends_with(file.file_name));
                assert!(file.byte_len > 0);
                assert_eq!(file.sha256.len(), 64);
                sum += file.byte_len;
            }
            assert_eq!(
                sum, set.total_bytes,
                "{}: total_bytes must equal the sum of its files",
                set.model_id
            );
            assert!(
                set.estimated_peak_bytes > set.total_bytes,
                "{}: peak residency must exceed artifact bytes (KV cache + Metal overhead)",
                set.model_id
            );
        }
    }

    #[test]
    fn mlx_sets_order_configs_before_shards() {
        // Download order is the array order; a bad revision must fail on a
        // small config file, never after gigabytes of shard transfer.
        for set in [&IOS_QWEN35_4B_MLX, &IOS_QWEN35_9B_MLX] {
            let first_shard = set
                .files
                .iter()
                .position(|file| file.file_name.ends_with(".safetensors"))
                .expect("every MLX set has at least one shard");
            let last_config = set
                .files
                .iter()
                .rposition(|file| !file.file_name.ends_with(".safetensors"))
                .expect("every MLX set has config files");
            assert!(
                last_config < first_shard,
                "{}: all non-shard files must precede the first shard",
                set.model_id
            );
        }
    }

    #[test]
    fn mlx_ids_resolve_and_ios_default_id_is_the_mlx_4b() {
        assert_eq!(
            mlx_artifact_set_for_id(IOS_QWEN35_4B_MLX_ID),
            Some(&IOS_QWEN35_4B_MLX)
        );
        assert_eq!(
            mlx_artifact_set_for_id(IOS_QWEN35_9B_MLX_ID),
            Some(&IOS_QWEN35_9B_MLX)
        );
        assert_eq!(mlx_artifact_set_for_id("qwen3.5-9b-q4-k-m"), None);
        assert_eq!(MobilePlatform::Ios.default_model_id(), IOS_QWEN35_4B_MLX_ID);
        assert_eq!(
            MobilePlatform::Android.default_model_id(),
            ANDROID_GEMMA4_E2B_LITERTLM_ID
        );
    }
}
