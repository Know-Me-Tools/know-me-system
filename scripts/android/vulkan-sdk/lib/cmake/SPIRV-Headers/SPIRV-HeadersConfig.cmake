# llama.cpp's Vulkan backend only needs a CONFIG package marker for the
# SPIR-V headers. Android's NDK ships the headers with shaderc, but does not
# install the upstream CMake package that desktop Vulkan SDKs provide.
# Resolve the include directory from the NDK selected by build.sh so the
# Android build remains reproducible without requiring a host Vulkan SDK.
set(_ndk_root "$ENV{ANDROID_NDK}")
if(NOT _ndk_root)
  set(_ndk_root "$ENV{ANDROID_NDK_ROOT}")
endif()

set(_spirv_headers_dir
    "${_ndk_root}/sources/third_party/shaderc/third_party/spirv-tools/external/spirv-headers/include")
if(NOT EXISTS "${_spirv_headers_dir}/spirv/1.0/spirv.h")
  message(FATAL_ERROR
    "Android NDK SPIR-V headers were not found under ${_spirv_headers_dir}")
endif()

if(NOT TARGET SPIRV-Headers::SPIRV-Headers)
  add_library(SPIRV-Headers::SPIRV-Headers INTERFACE IMPORTED)
  set_target_properties(SPIRV-Headers::SPIRV-Headers PROPERTIES
    INTERFACE_INCLUDE_DIRECTORIES "${_spirv_headers_dir}")
endif()

set(SPIRV-Headers_FOUND TRUE)
