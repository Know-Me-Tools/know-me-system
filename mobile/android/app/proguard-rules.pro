# KnowMe Android keep rules.
#
# R8 runs on the release variant and shrinks classes it cannot prove are
# reachable. The LiteRT-LM bridge is reached ONLY from Rust via JNI reflection
# (gen_ui_inference resolves it by fully-qualified name in
# android_litert_jni.rs), so static analysis sees no reference to it and shrinks
# it away — release builds then fail at load with
# `ClassNotFoundException: tools.knowme.knowme.KnowMeLiteRtLmBridge`, while
# debug builds work because R8 does not run there.
#
# Keep the class AND its members: Rust resolves individual methods reflectively
# after loading the class, so member-level shrinking or renaming breaks the
# bridge just as surely as removing the class.
-keep class tools.knowme.knowme.KnowMeLiteRtLmBridge { *; }
-keep class tools.knowme.knowme.KnowMeLiteRtLmBridge$* { *; }

# The bridge binds Google's official LiteRT-LM Android API. Its types cross the
# JNI boundary and are also used reflectively by the runtime, so keep the
# published surface rather than letting R8 rename or drop parts of it.
-keep class com.google.ai.edge.litertlm.** { *; }
-dontwarn com.google.ai.edge.litertlm.**
