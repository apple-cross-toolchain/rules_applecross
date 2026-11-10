// TARGET_OS_* for shaders that include Apple's TargetConditionals.h; the
// driver passes the target as __MSL_TARGET_*.
#pragma once
#if defined(__MSL_TARGET_MACOS)
#define TARGET_OS_OSX 1
#define TARGET_OS_MAC 1
#define TARGET_OS_IPHONE 0
#define TARGET_OS_IOS 0
#elif defined(__MSL_TARGET_IOS) || defined(__MSL_TARGET_TVOS) || defined(__MSL_TARGET_XROS)
#define TARGET_OS_OSX 0
#define TARGET_OS_MAC 1
#define TARGET_OS_IPHONE 1
#define TARGET_OS_IOS defined(__MSL_TARGET_IOS)
#define TARGET_OS_TV defined(__MSL_TARGET_TVOS)
#define TARGET_OS_VISION defined(__MSL_TARGET_XROS)
#endif
#ifndef TARGET_OS_TV
#define TARGET_OS_TV 0
#endif
#ifndef TARGET_OS_VISION
#define TARGET_OS_VISION 0
#endif
#define TARGET_OS_WATCH 0
#ifdef __MSL_TARGET_SIMULATOR
#define TARGET_OS_SIMULATOR 1
#else
#define TARGET_OS_SIMULATOR 0
#endif
