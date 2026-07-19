//! Android JNI binding for the official LiteRT-LM Android API.
//!
//! Rust still owns model selection, context construction, tool policy, and
//! event normalization. This module only bridges into the app-local
//! `tools.knowme.knowme.KnowMeLiteRtLmBridge` adapter, which binds Google's
//! Kotlin/Android LiteRT-LM package.

use std::path::Path;
use std::sync::OnceLock;

use gen_ui_types::error::{CoreError, CoreResult};
use gen_ui_types::inference::{InferenceRequest, InferenceToolResult};
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue};
use jni::{JNIEnv, JavaVM};
use serde::Deserialize;

static JAVA_VM: OnceLock<JavaVM> = OnceLock::new();
static BRIDGE_CLASS: OnceLock<GlobalRef> = OnceLock::new();

pub fn set_java_vm(vm: JavaVM) {
    let _ = JAVA_VM.set(vm);
}

pub fn java_vm_available() -> bool {
    JAVA_VM.get().is_some()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeLiteRtLmResponse {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub tool_calls: Vec<NativeLiteRtLmToolCall>,
    #[serde(default)]
    pub tool_result_fallback_applied: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NativeLiteRtLmToolCall {
    pub name: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

pub struct NativeLiteRtLmAdapter;

impl NativeLiteRtLmAdapter {
    pub fn load(model_path: &Path, cache_dir: &Path) -> CoreResult<Self> {
        let mut env = attach_env("LiteRT-LM JavaVM")?;
        let model_path = env
            .new_string(model_path.to_string_lossy().as_ref())
            .map_err(|error| jni_error(&mut env, "LiteRT-LM model path string", error))?;
        let cache_dir = env
            .new_string(cache_dir.to_string_lossy().as_ref())
            .map_err(|error| jni_error(&mut env, "LiteRT-LM cache dir string", error))?;
        call_static_string(
            &mut env,
            "load",
            "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            &[
                JValue::Object(&JObject::from(model_path)),
                JValue::Object(&JObject::from(cache_dir)),
            ],
        )?;
        Ok(Self)
    }

    pub fn generate_text(&self, request: &InferenceRequest) -> CoreResult<NativeLiteRtLmResponse> {
        let mut env = attach_env("LiteRT-LM JavaVM")?;
        let request_json = serde_json::to_string(request)
            .map_err(|error| CoreError::Serde(format!("LiteRT-LM request JSON: {error}")))?;
        let request_json = env
            .new_string(request_json)
            .map_err(|error| jni_error(&mut env, "LiteRT-LM request string", error))?;
        let response = call_static_string(
            &mut env,
            "sendMessage",
            "(Ljava/lang/String;)Ljava/lang/String;",
            &[JValue::Object(&JObject::from(request_json))],
        )?;
        serde_json::from_str(&response)
            .map_err(|error| CoreError::Serde(format!("LiteRT-LM response JSON: {error}")))
    }

    pub fn continue_with_tool_results(
        &self,
        results: &[InferenceToolResult],
    ) -> CoreResult<NativeLiteRtLmResponse> {
        let mut env = attach_env("LiteRT-LM JavaVM")?;
        let results_json = serde_json::to_string(results)
            .map_err(|error| CoreError::Serde(format!("LiteRT-LM tool results JSON: {error}")))?;
        let results_json = env
            .new_string(results_json)
            .map_err(|error| jni_error(&mut env, "LiteRT-LM tool results string", error))?;
        let response = call_static_string(
            &mut env,
            "sendToolResults",
            "(Ljava/lang/String;)Ljava/lang/String;",
            &[JValue::Object(&JObject::from(results_json))],
        )?;
        serde_json::from_str(&response)
            .map_err(|error| CoreError::Serde(format!("LiteRT-LM response JSON: {error}")))
    }

    pub fn cancel() {
        if let Ok(mut env) = attach_env("LiteRT-LM JavaVM") {
            let _ = call_static_void(&mut env, "cancel", "()V", &[]);
        }
    }

    pub fn close() {
        if let Ok(mut env) = attach_env("LiteRT-LM JavaVM") {
            let _ = call_static_void(&mut env, "close", "()V", &[]);
        }
    }
}

fn attach_env(context: &str) -> CoreResult<jni::AttachGuard<'static>> {
    JAVA_VM
        .get()
        .ok_or_else(|| CoreError::Terminal(format!("{context}: JNI_OnLoad has not run")))?
        .attach_current_thread()
        .map_err(|error| CoreError::Terminal(format!("{context}: {error}")))
}

fn call_static_string(
    env: &mut JNIEnv<'_>,
    method: &str,
    signature: &str,
    args: &[JValue<'_, '_>],
) -> CoreResult<String> {
    let bridge_class = resolve_bridge_class(env)?;
    let value = env
        .call_static_method(bridge_class, method, signature, args)
        .map_err(|error| jni_error(env, &format!("LiteRT-LM bridge {method}"), error))?;
    let object = value
        .l()
        .map_err(|error| jni_error(env, &format!("LiteRT-LM bridge {method} result"), error))?;
    let text = JString::from(object);
    env.get_string(&text)
        .map(|value| value.to_string_lossy().into_owned())
        .map_err(|error| jni_error(env, &format!("LiteRT-LM bridge {method} string"), error))
}

fn call_static_void(
    env: &mut JNIEnv<'_>,
    method: &str,
    signature: &str,
    args: &[JValue<'_, '_>],
) -> CoreResult<()> {
    let bridge_class = resolve_bridge_class(env)?;
    env.call_static_method(bridge_class, method, signature, args)
        .map(|_| ())
        .map_err(|error| jni_error(env, &format!("LiteRT-LM bridge {method}"), error))
}

fn resolve_bridge_class<'env>(env: &mut JNIEnv<'env>) -> CoreResult<JClass<'env>> {
    if let Some(global) = BRIDGE_CLASS.get() {
        let local = env
            .new_local_ref(global.as_obj())
            .map_err(|error| jni_error(env, "LiteRT-LM bridge cached class", error))?;
        return Ok(JClass::from(local));
    }

    let activity_thread = env
        .find_class("android/app/ActivityThread")
        .map_err(|error| jni_error(env, "LiteRT-LM ActivityThread class", error))?;
    let application = env
        .call_static_method(
            activity_thread,
            "currentApplication",
            "()Landroid/app/Application;",
            &[],
        )
        .map_err(|error| jni_error(env, "LiteRT-LM current application", error))?
        .l()
        .map_err(|error| jni_error(env, "LiteRT-LM current application object", error))?;
    if application.is_null() {
        return Err(CoreError::Terminal(
            "LiteRT-LM current application is not available".into(),
        ));
    }
    let class_loader = env
        .call_method(
            &application,
            "getClassLoader",
            "()Ljava/lang/ClassLoader;",
            &[],
        )
        .map_err(|error| jni_error(env, "LiteRT-LM application class loader", error))?
        .l()
        .map_err(|error| jni_error(env, "LiteRT-LM application class loader object", error))?;
    let class_name = env
        .new_string("tools.knowme.knowme.KnowMeLiteRtLmBridge")
        .map_err(|error| jni_error(env, "LiteRT-LM bridge class name", error))?;
    let class_name = JObject::from(class_name);
    let class = env
        .call_method(
            &class_loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&class_name)],
        )
        .map_err(|error| jni_error(env, "LiteRT-LM bridge loadClass", error))?
        .l()
        .map_err(|error| jni_error(env, "LiteRT-LM bridge class object", error))?;
    let global = env
        .new_global_ref(&class)
        .map_err(|error| jni_error(env, "LiteRT-LM bridge global class", error))?;
    let _ = BRIDGE_CLASS.set(global);
    let local = env
        .new_local_ref(
            BRIDGE_CLASS
                .get()
                .ok_or_else(|| CoreError::Terminal("LiteRT-LM bridge class cache failed".into()))?
                .as_obj(),
        )
        .map_err(|error| jni_error(env, "LiteRT-LM bridge local class", error))?;
    Ok(JClass::from(local))
}

fn jni_error(env: &mut JNIEnv<'_>, context: &str, error: jni::errors::Error) -> CoreError {
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_describe();
        let _ = env.exception_clear();
    }
    CoreError::Terminal(format!("{context}: {error}"))
}
