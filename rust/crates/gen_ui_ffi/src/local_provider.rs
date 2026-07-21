//! Platform-leaf local model registrations for the embedded UAR.
//!
//! The immutable artifact catalog belongs to `gen_ui_inference`; UAR provider
//! semantics belong to `gen_ui_agent`.  Keeping this tiny composition in the
//! FFI leaf preserves the shared `InferenceProvider` boundary while ensuring
//! the exact artifact selected for each platform is the model UAR exposes to
//! agents.

#[cfg(any(target_os = "android", test))]
pub(crate) fn android_litert_lm_uar_provider_config() -> gen_ui_agent::uar::UarProviderConfig {
    gen_ui_agent::uar::android_litert_lm_provider_config(
        gen_ui_inference::catalog::ANDROID_GEMMA4_E2B_LITERTLM_ID,
    )
}

#[cfg(any(target_os = "ios", test))]
pub(crate) fn ios_qwen35_metal_uar_provider_config() -> gen_ui_agent::uar::UarProviderConfig {
    gen_ui_agent::uar::ios_qwen35_metal_provider_config(
        gen_ui_inference::catalog::IPHONE_QWEN35_9B_ID,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn android_uar_registration_uses_the_pinned_litert_artifact() {
        let provider = android_litert_lm_uar_provider_config();
        let expected = gen_ui_inference::catalog::ANDROID_GEMMA4_E2B_LITERTLM_ID;

        assert_eq!(provider.default_model.as_deref(), Some(expected));
        assert_eq!(provider.models.len(), 1);
        assert_eq!(provider.models[0].id, expected);
        assert!(
            gen_ui_inference::catalog::artifact_set_for_id(&provider.models[0].id).is_some(),
            "UAR must never advertise an Android local model without a pinned artifact"
        );
    }

    #[test]
    fn ios_uar_registration_uses_the_pinned_model_and_projector() {
        let provider = ios_qwen35_metal_uar_provider_config();
        let expected = gen_ui_inference::catalog::IPHONE_QWEN35_9B_ID;

        assert_eq!(provider.default_model.as_deref(), Some(expected));
        assert_eq!(provider.models.len(), 1);
        assert_eq!(provider.models[0].id, expected);
        assert!(provider.models[0].supports_vision);

        let artifacts = gen_ui_inference::catalog::artifact_set_for_id(&provider.models[0].id)
            .expect("UAR must never advertise an iOS local model without pinned artifacts");
        assert!(
            artifacts.projector.is_some(),
            "a vision-capable iOS registration must include its pinned MTMD projector"
        );
    }
}
