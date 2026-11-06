// Copyright 2017 The Bazel Authors. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// wrapped_clang.cc: Pass args to 'xcrun clang' and zip dsym files.
//
// wrapped_clang passes its args to clang, but also supports a separate set of
// invocations to generate dSYM files.  If "LINKED_BINARY" and/or "DSYM_PATH"
// flags are passed in, they are used to construct that separate set of
// invocations (instead of being passed to clang).
// "LINKED_BINARY": Workspace-relative path to binary output of the
//    link action generating the dsym file.
// "DSYM_PATH": Workspace-relative path to dSYM dwarf file.

#include <libgen.h>
#include <limits.h>
#include <spawn.h>
#include <string.h>
#include <stdlib.h>
#include <sys/wait.h>
#include <unistd.h>

#include <cerrno>
#include <climits>
#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <functional>
#include <iostream>
#include <map>
#include <memory>
#include <sstream>
#include <utility>
#include <vector>

extern char **environ;

namespace {

constexpr char kAddASTPathPrefix[] = "-Wl,-add_ast_path,";

// Where Xcode is on a Mac by default: a Linux-built binary's run paths into
// Xcode name it.
constexpr char kInstalledDeveloperDir[] = "/Applications/Xcode.app/Contents/Developer";

// Returns the base name of the given filepath. For example, given
// /foo/bar/baz.txt, returns 'baz.txt'.
const char *Basename(const char *filepath) {
  const char *base = strrchr(filepath, '/');
  return base ? (base + 1) : filepath;
}

// Returns the directory name of the given filepath. For example, given
// "/foo/bar/baz.txt", returns "/foo/bar".
std::string Dirname(const std::string &path) {
  auto last_slash = path.rfind('/');
  if (last_slash == std::string::npos) {
    return std::string();
  }
  return path.substr(0, last_slash);
}

#if !__APPLE__
std::string ResolveFromAncestor(const std::string &path,
                                const std::string &cwd) {
  if (path.empty() || path[0] == '/' || access(path.c_str(), F_OK) == 0) {
    return path;
  }
  for (std::string dir = Dirname(cwd); !dir.empty(); dir = Dirname(dir)) {
    std::string candidate = dir + "/" + path;
    if (access(candidate.c_str(), F_OK) == 0) {
      return candidate;
    }
  }
  return path;
}
#endif

bool ContainsArgument(const std::vector<std::string> &args,
                      const std::string &value) {
  for (const auto &arg : args) {
    if (arg == value) {
      return true;
    }
  }
  return false;
}

// Unescape and unquote an argument read from a line of a response file.
static std::string Unescape(const std::string &arg) {
  std::string result;
  auto length = arg.size();
  for (size_t i = 0; i < length; ++i) {
    auto ch = arg[i];

    // If it's a backslash, consume it and append the character that follows.
    if (ch == '\\' && i + 1 < length) {
      ++i;
      result.push_back(arg[i]);
      continue;
    }

    // If it's a quote, process everything up to the matching quote, unescaping
    // backslashed characters as needed.
    if (ch == '"' || ch == '\'') {
      auto quote = ch;
      ++i;
      while (i != length && arg[i] != quote) {
        if (arg[i] == '\\' && i + 1 < length) {
          ++i;
        }
        result.push_back(arg[i]);
        ++i;
      }
      if (i == length) {
        break;
      }
      continue;
    }

    // It's a regular character.
    result.push_back(ch);
  }

  return result;
}

// Converts an array of string arguments to char *arguments.
// Note that the lifetime of the char* arguments in the returned array
// are controlled by the lifetime of the strings in args.
std::vector<const char *> ConvertToCArgs(const std::vector<std::string> &args) {
  std::vector<const char *> c_args;
#if __APPLE__
  c_args.push_back(Basename(args[0].c_str()));
#else
  c_args.push_back(args[0].c_str());
#endif
  for (int i = 1; i < args.size(); i++) {
    c_args.push_back(args[i].c_str());
  }
  c_args.push_back(nullptr);
  return c_args;
}

// Spawns a subprocess for given arguments args. The first argument is used
// for the executable path. Returns true on success, false on failure.
bool RunSubProcess(const std::vector<std::string> &args) {
  std::vector<const char *> exec_argv = ConvertToCArgs(args);
  pid_t pid;
  int status = posix_spawn(&pid, args[0].c_str(), nullptr, nullptr,
                           const_cast<char **>(exec_argv.data()), environ);
  if (status == 0) {
    int wait_status;
    do {
      wait_status = waitpid(pid, &status, 0);
    } while ((wait_status == -1) && (errno == EINTR));
    if (wait_status < 0) {
      std::cerr << "Error waiting on child process '" << args[0] << "'. "
                << strerror(errno) << "\n";
      return false;
    }
    if (WIFEXITED(status)) {
      if (WEXITSTATUS(status) != 0) {
        std::cerr << "Error in child process '" << args[0] << "'. "
                  << WEXITSTATUS(status) << "\n";
        return false;
      }
    } else if (WIFSIGNALED(status)) {
      std::cerr << "Error in child process '" << args[0]
                << "'. Terminated by signal " << WTERMSIG(status) << "\n";
      return false;
    }
  } else {
    std::cerr << "Error forking process '" << args[0] << "'. "
              << strerror(status) << "\n";
    return false;
  }
  return true;
}

// Finds and replaces all instances of oldsub with newsub, in-place on str.
void FindAndReplace(const std::string &oldsub, const std::string &newsub,
                    std::string *str) {
  int start = 0;
  while ((start = str->find(oldsub, start)) != std::string::npos) {
    str->replace(start, oldsub.length(), newsub);
    start += newsub.length();
  }
}

// If arg is of the classic flag form "foo=bar", and flagname is 'foo', sets
// str to point to a new std::string 'bar' and returns true.
// Otherwise, returns false.
bool SetArgIfFlagPresent(const std::string &arg, const std::string &flagname,
                         std::string *str) {
  std::string prefix_string = flagname + "=";
  if (arg.compare(0, prefix_string.length(), prefix_string) == 0) {
    *str = arg.substr(prefix_string.length());
    return true;
  }
  return false;
}

// Returns the DEVELOPER_DIR environment variable in the current process
// environment. Aborts if this variable is unset.
std::string GetMandatoryEnvVar(const std::string &var_name) {
  char *env_value = getenv(var_name.c_str());
  if (env_value == nullptr) {
    std::cerr << "Error: " << var_name << " not set.\n";
    abort();
  }
  return env_value;
}

// Returns the requested environment variable in the current process's
// environment, or an empty string if this variable is unset.
static std::string GetEnvVar(const std::string &var_name) {
  char *env_value = getenv(var_name.c_str());
  if (env_value == nullptr) {
    return "";
  }
  return env_value;
}

// Returns true if `str` starts with the specified `prefix`.
bool StartsWith(const std::string &str, const std::string &prefix) {
  return str.compare(0, prefix.size(), prefix) == 0;
}

// If *`str` begins `prefix`, strip it out and return true.
// Otherwise leave *`str` unchanged and return false.
bool StripPrefixStringIfPresent(std::string *str, const std::string &prefix) {
  if (StartsWith(*str, prefix)) {
    *str = str->substr(prefix.size());
    return true;
  }
  return false;
}

// An RAII temporary file.
class TempFile {
 public:
  // Create a new temporary file using the given path template string (the same
  // form used by `mkstemp`). The file will automatically be deleted when the
  // object goes out of scope.
  static std::unique_ptr<TempFile> Create(const std::string &path_template) {
    const char *tmpDir = getenv("TMPDIR");
    if (!tmpDir) {
      tmpDir = "/tmp";
    }
    size_t size = strlen(tmpDir) + path_template.size() + 2;
    std::unique_ptr<char[]> path(new char[size]);
    snprintf(path.get(), size, "%s/%s", tmpDir, path_template.c_str());

    if (mkstemp(path.get()) == -1) {
      std::cerr << "Failed to create temporary file '" << path.get()
                << "': " << strerror(errno) << "\n";
      return nullptr;
    }
    return std::unique_ptr<TempFile>(new TempFile(path.get()));
  }

  // Explicitly make TempFile non-copyable and movable.
  TempFile(const TempFile &) = delete;
  TempFile &operator=(const TempFile &) = delete;
  TempFile(TempFile &&) = default;
  TempFile &operator=(TempFile &&) = default;

  ~TempFile() { remove(path_.c_str()); }

  // Gets the path to the temporary file.
  std::string GetPath() const { return path_; }

 private:
  explicit TempFile(const std::string &path) : path_(path) {}

  std::string path_;
};

static std::unique_ptr<TempFile> WriteResponseFile(
    const std::vector<std::string> &args) {
  auto response_file = TempFile::Create("wrapped_clang_params.XXXXXX");
  std::ofstream response_file_stream(response_file->GetPath());

  for (const auto &arg : args) {
    // When Clang/Swift write out a response file to communicate from driver to
    // frontend, they just quote every argument to be safe; we duplicate that
    // instead of trying to be "smarter" and only quoting when necessary.
    response_file_stream << '"';
    for (auto ch : arg) {
      if (ch == '"' || ch == '\\') {
        response_file_stream << '\\';
      }
      response_file_stream << ch;
    }
    response_file_stream << "\"\n";
  }

  response_file_stream.close();
  return response_file;
}

void ProcessArgument(const std::string arg, const std::string developer_dir,
                     const std::string sdk_root, const std::string cwd,
                     bool relative_ast_path, bool &strip_debug_symbols,
                     std::string &linked_binary,
                     std::string &dsym_path,
                     std::function<void(const std::string &)> consumer);

bool ProcessResponseFile(const std::string arg, const std::string developer_dir,
                         const std::string sdk_root, const std::string cwd,
                         bool relative_ast_path, bool &strip_debug_symbols,
                         std::string &linked_binary,
                         std::string &dsym_path,
                         std::function<void(const std::string &)> consumer) {
  auto path = arg.substr(1);
  std::ifstream original_file(path);
  // Ignore non-file args such as '@loader_path/...'
  if (!original_file.good()) {
    return false;
  }

  std::string arg_from_file;
  while (std::getline(original_file, arg_from_file)) {
    // Arguments in response files might be quoted/escaped, so we need to
    // unescape them ourselves.
    ProcessArgument(Unescape(arg_from_file), developer_dir, sdk_root, cwd,
                    relative_ast_path, strip_debug_symbols, linked_binary,
                    dsym_path, consumer);
  }

  return true;
}

std::string GetCurrentDirectory() {
  // Passing null,0 causes getcwd to allocate the buffer of the correct size.
  char *buffer = getcwd(nullptr, 0);
  std::string cwd(buffer);
  free(buffer);
  return cwd;
}

void ProcessArgument(const std::string arg, const std::string developer_dir,
                     const std::string sdk_root, const std::string cwd,
                     bool relative_ast_path, bool &strip_debug_symbols,
                     std::string &linked_binary,
                     std::string &dsym_path,
                     std::function<void(const std::string &)> consumer) {
  auto new_arg = arg;
  if (arg[0] == '@') {
    if (ProcessResponseFile(arg, developer_dir, sdk_root, cwd,
                            relative_ast_path, strip_debug_symbols,
                            linked_binary, dsym_path, consumer)) {
      return;
    }
  }

  if (SetArgIfFlagPresent(arg, "LINKED_BINARY", &linked_binary)) {
    return;
  }
  if (SetArgIfFlagPresent(arg, "DSYM_PATH", &dsym_path)) {
    return;
  }
  if (arg == "STRIP_DEBUG_SYMBOLS") {
    strip_debug_symbols = true;
    return;
  }

  // A run path names where the binary finds its libraries when it runs, on
  // a Mac: off a Mac, DEVELOPER_DIR is the staged tree (a relative path), so
  // a run path into it gets the installed Xcode's instead, as a Mac with
  // Xcode in its usual place gives (a test's XCTest.framework). Every other
  // use keeps the staged tree, which the build reads.
  if (!developer_dir.empty() && developer_dir[0] != '/' &&
      StartsWith(new_arg, "-Wl,-rpath,")) {
    FindAndReplace("__BAZEL_XCODE_DEVELOPER_DIR__", kInstalledDeveloperDir,
                   &new_arg);
  }
  FindAndReplace("__BAZEL_XCODE_DEVELOPER_DIR__", developer_dir, &new_arg);
  FindAndReplace("__BAZEL_XCODE_SDKROOT__", sdk_root, &new_arg);
  FindAndReplace("__BAZEL_EXECUTION_ROOT__", cwd, &new_arg);

  // Make the `add_ast_path` options used to embed Swift module references
  // absolute to enable Swift debugging without dSYMs: see
  // https://forums.swift.org/t/improving-swift-lldb-support-for-path-remappings/22694
  if (!relative_ast_path &&
      StripPrefixStringIfPresent(&new_arg, kAddASTPathPrefix)) {
    // Only modify relative paths.
    if (!StartsWith(arg, "/")) {
      new_arg = std::string(kAddASTPathPrefix) + cwd + "/" + new_arg;
    } else {
      new_arg = std::string(kAddASTPathPrefix) + new_arg;
    }
  }

  consumer(new_arg);
}

// Driver flags Xcode's clang accepts and LLVM's doesn't: those of Xcode
// 27.2's clang (Apple clang 21) that LLVM 23.1.2's clang rejects as unknown
// (tools/apple_clang_flags.py lists them again). Off a Mac, where this runs
// LLVM's clang, a build with one fails here, since dropping it can change
// the output from Xcode's (-fobjc-relative-method-lists, Objective-C method
// lists' layout); --config=relax_clang_apple_flags
// (RULES_APPLECROSS_RELAX_APPLE_CLANG_FLAGS=1) drops them with a warning.
enum AppleFlagKind { kFlag, kJoined, kSeparate };
struct AppleFlag {
  const char *name;
  AppleFlagKind kind;  // kJoined: a prefix with its value; kSeparate: values follow.
  int values;          // kSeparate's: how many arguments follow.
};
const AppleFlag kAppleOnlyFlags[] = {
    {"--emit-source-metadata=", kJoined, 0},
    {"--source-metadata-list=", kJoined, 0},
    {"-fapinotes-cache-path=", kJoined, 0},
    {"-fbounds-safety", kFlag, 0},
    {"-fbounds-safety-adoption-mode", kFlag, 0},
    {"-fbounds-safety-bringup-missing-checks", kFlag, 0},
    {"-fbounds-safety-bringup-missing-checks=", kJoined, 0},
    {"-fbounds-safety-debug-trap-reasons=", kJoined, 0},
    {"-fbounds-safety-internal-default-to-legacy-checks", kFlag, 0},
    {"-fbounds-safety-relaxed-system-headers", kFlag, 0},
    {"-fbounds-safety-soft-traps=", kJoined, 0},
    {"-fbounds-safety-unique-traps", kFlag, 0},
    {"-fbranch-target-identification", kFlag, 0},
    {"-fbranch-target-identification-enhanced", kFlag, 0},
    {"-fchecked-pointer-arithmetic", kFlag, 0},
    {"-fchecked-pointer-arithmetic=", kJoined, 0},
    {"-fdepscan", kFlag, 0},
    {"-fdepscan-daemon=", kJoined, 0},
    {"-fdepscan-prefix-map", kSeparate, 2},
    {"-fdepscan-prefix-map-sdk=", kJoined, 0},
    {"-fdepscan-prefix-map-toolchain=", kJoined, 0},
    {"-fdepscan-prefix-map=", kJoined, 0},
    {"-fdepscan-share-identifier", kSeparate, 1},
    {"-fdepscan-share-parent", kFlag, 0},
    {"-fdepscan-share-parent=", kJoined, 0},
    {"-fdepscan-share-stop=", kJoined, 0},
    {"-fdepscan-share=", kJoined, 0},
    {"-fdepscan=", kJoined, 0},
    {"-fdiagnostics-add-output=", kJoined, 0},
    {"-fdisable-objc-method-tracing", kFlag, 0},
    {"-fdisable-system-symbol-reference-tracing", kFlag, 0},
    {"-fexperimental-bounds-safety-attributes", kFlag, 0},
    {"-fgeneric-block-helpers", kFlag, 0},
    {"-fgeneric-block-helpers-use-extended-layout", kFlag, 0},
    {"-fin-descriptor-block-flags", kFlag, 0},
    {"-finit-global-zero", kFlag, 0},
    {"-flto-override-mcpu-no-error", kFlag, 0},
    {"-flto-override-mcpu=", kJoined, 0},
    {"-flto-override-mtune=", kJoined, 0},
    {"-fmodule-related-to-pch", kFlag, 0},
    {"-fno-bounds-safety", kFlag, 0},
    {"-fno-bounds-safety-adoption-mode", kFlag, 0},
    {"-fno-bounds-safety-bringup-missing-checks", kFlag, 0},
    {"-fno-bounds-safety-bringup-missing-checks=", kJoined, 0},
    {"-fno-bounds-safety-internal-default-to-legacy-checks", kFlag, 0},
    {"-fno-bounds-safety-relaxed-system-headers", kFlag, 0},
    {"-fno-bounds-safety-unique-traps", kFlag, 0},
    {"-fno-checked-pointer-arithmetic", kFlag, 0},
    {"-fno-depscan-share", kFlag, 0},
    {"-fno-experimental-bounds-safety-attributes", kFlag, 0},
    {"-fno-generic-block-helpers", kFlag, 0},
    {"-fno-generic-block-helpers-use-extended-layout", kFlag, 0},
    {"-fno-in-descriptor-block-flags", kFlag, 0},
    {"-fno-init-global-zero", kFlag, 0},
    {"-fno-modules-check-relocated", kFlag, 0},
    {"-fno-ptrauth-abi-version", kFlag, 0},
    {"-fno-ptrauth-kernel-abi-version", kFlag, 0},
    {"-fno-target-arch-variant-auto-multi-arch", kFlag, 0},
    {"-fno-trap-function-returns", kFlag, 0},
    {"-fno-typed-cxx-delete", kFlag, 0},
    {"-fno-typed-cxx-new-delete", kFlag, 0},
    {"-fno-typed-memory-operations-experimental", kFlag, 0},
    {"-fno-unique-traps", kFlag, 0},
    {"-fobjc-relative-method-lists", kFlag, 0},
    {"-fptrauth-abi-version=", kJoined, 0},
    {"-fptrauth-kernel-abi-version", kFlag, 0},
    {"-fptrauth-objc-isa-masking", kFlag, 0},
    {"-fptrauth-objc-isa-mode=", kJoined, 0},
    {"-fptrauth-returns=", kJoined, 0},
    {"-fptrauth-soft", kFlag, 0},
    {"-fsmall-block-descriptors", kFlag, 0},
    {"-fsuppress-conflicting-types", kFlag, 0},
    {"-ftarget-arch-variant-auto-multi-arch", kFlag, 0},
    {"-ftarget-arch-variant-mode=", kJoined, 0},
    {"-ftarget-variant-availability-checks", kFlag, 0},
    {"-ftrap-function-returns", kFlag, 0},
    {"-ftrivial-auto-var-init-skip-cpp-obj", kFlag, 0},
    {"-ftrivial-auto-var-init-skip-non-pod", kFlag, 0},
    {"-ftrivial-auto-var-init-skip-non-ptr-array", kFlag, 0},
    {"-ftrivial-auto-var-init-skip-stl", kFlag, 0},
    {"-ftrivial-auto-var-zero-init-max-size=", kJoined, 0},
    {"-ftyped-cxx-delete", kFlag, 0},
    {"-ftyped-cxx-new-delete", kFlag, 0},
    {"-ftyped-memory-operations-experimental", kFlag, 0},
    {"-funique-traps", kFlag, 0},
    {"-greproducible", kFlag, 0},
    {"-iapinotes-path", kSeparate, 1},
    {"-index-ignore-macros", kFlag, 0},
    {"-index-ignore-pcms", kFlag, 0},
    {"-index-ignore-system-symbols", kFlag, 0},
    {"-index-record-codegen-name", kFlag, 0},
    {"-index-store-compress", kFlag, 0},
    {"-index-store-path", kSeparate, 1},
    {"-index-unit-output-path", kSeparate, 1},
    {"-ivfsstatcache", kSeparate, 1},
    {"-iworkdir=", kJoined, 0},
    {"-mno-implicit-sme", kFlag, 0},
    {"-target-arch-variant", kSeparate, 1},
};

// The Apple-only flag `arg` is, if any.
const AppleFlag *AppleOnlyFlag(const std::string &arg) {
  for (const AppleFlag &flag : kAppleOnlyFlags) {
    if (flag.kind == kJoined ? StartsWith(arg, flag.name) : arg == flag.name) {
      return &flag;
    }
  }
  return nullptr;
}

}  // namespace

int main(int argc, char *argv[]) {
  // If HEADER_PARSING_OUTPUT is set, create an empty file at that path.
  const char *header_parsing_output = getenv("HEADER_PARSING_OUTPUT");
  if (header_parsing_output) {
    std::ofstream hp_file(header_parsing_output);
    hp_file.close();
  }

  std::string tool_name;

  std::string binary_name = Basename(argv[0]);
  if (binary_name == "wrapped_clang_pp") {
    tool_name = "clang++";
  } else if (binary_name == "wrapped_clang") {
    tool_name = "clang";
  } else {
    std::cerr << "Binary must either be named 'wrapped_clang' or "
                 "'wrapped_clang_pp', not "
              << binary_name << "\n";
    abort();
  }

  const std::string cwd = GetCurrentDirectory();
#if __APPLE__
  std::string developer_dir = GetMandatoryEnvVar("DEVELOPER_DIR");
  // Bazel sets SDKROOT to the Mac's Xcode; APPLECROSS_SYSROOT overrides it.
  std::string sdk_root = GetEnvVar("APPLECROSS_SYSROOT");
  if (sdk_root.empty()) {
    sdk_root = GetMandatoryEnvVar("SDKROOT");
  }
#else
  // On non-Apple platforms, get DEVELOPER_DIR from the environment (set by
  // the CC toolchain config's apple_env feature).
  std::string developer_dir = GetEnvVar("DEVELOPER_DIR");
  if (developer_dir == "") {
    // Fallback: compute from our own binary location.
    // wrapped_clang lives in the toolchain repo root, developer_dir is
    // under Xcode.app/Contents/Developer relative to that.
    char exe_buf[PATH_MAX];
    ssize_t len = readlink("/proc/self/exe", exe_buf, sizeof(exe_buf) - 1);
    if (len > 0) {
      exe_buf[len] = '\0';
      developer_dir = Dirname(std::string(exe_buf)) +
                       "/Xcode.app/Contents/Developer";
    }
  }
  if (developer_dir == "") {
    std::cerr << "Error: could not find active developer directory. "
              << "Set DEVELOPER_DIR.\n";
    abort();
  }
  developer_dir = ResolveFromAncestor(developer_dir, cwd);

  std::string sdk_root = GetEnvVar("SDKROOT");
  if (sdk_root == "") {
    sdk_root = GetEnvVar("APPLECROSS_SYSROOT");
  }
  if (sdk_root == "") {
    std::string sdk_platform = GetMandatoryEnvVar("APPLE_SDK_PLATFORM");
    std::string sdk_version = GetMandatoryEnvVar("APPLE_SDK_VERSION_OVERRIDE");
    sdk_root = developer_dir + "/Platforms/" + sdk_platform +
        ".platform/Developer/SDKs/" + sdk_platform + sdk_version + ".sdk";
  }
  sdk_root = ResolveFromAncestor(sdk_root, cwd);
#endif
  std::string linked_binary, dsym_path;
  bool strip_debug_symbols = false;

  // Set by macOS execution platforms to use the staged LLVM instead of xcrun.
  std::string toolchain_bin = GetEnvVar("APPLECROSS_TOOLCHAIN_BIN");
#if __APPLE__
  bool use_xcrun = toolchain_bin.empty();
#else
  bool use_xcrun = false;
  if (toolchain_bin.empty()) {
    toolchain_bin = developer_dir +
        "/Toolchains/XcodeDefault.xctoolchain/usr/bin";
  }
#endif
  std::vector<std::string> invocation_args;
  if (use_xcrun) {
    invocation_args = {"/usr/bin/xcrun", tool_name};
  } else {
    setenv("SDKROOT", sdk_root.c_str(), 1);
    invocation_args = {toolchain_bin + "/" + tool_name};
  }
  std::vector<std::string> processed_args = {};

  bool relative_ast_path = getenv("RELATIVE_AST_PATH") != nullptr;
  bool relax_apple_flags =
      GetEnvVar("RULES_APPLECROSS_RELAX_APPLE_CLANG_FLAGS") == "1";
  std::vector<std::string> refused_flags;
  int skip_values = 0;
  auto consumer = [&](const std::string &arg) {
    if (skip_values > 0) {
      --skip_values;
      return;
    }
    if (!use_xcrun) {
      if (const AppleFlag *flag = AppleOnlyFlag(arg)) {
        if (!relax_apple_flags) {
          refused_flags.push_back(arg);
          return;
        }
        std::cerr << "warning: dropping " << arg
                  << ", which only Xcode's clang accepts; the output may "
                     "differ from Xcode's (--config=relax_clang_apple_flags)\n";
        skip_values = flag->kind == kSeparate ? flag->values : 0;
        return;
      }
    }
    processed_args.push_back(arg);
  };
  for (int i = 1; i < argc; i++) {
    std::string arg(argv[i]);

    ProcessArgument(arg, developer_dir, sdk_root, cwd, relative_ast_path,
                    strip_debug_symbols, linked_binary, dsym_path, consumer);
  }

  if (!refused_flags.empty()) {
    for (const std::string &flag : refused_flags) {
      std::cerr << "error: " << flag
                << " is a flag only Xcode's clang accepts, which LLVM's clang "
                   "off a Mac rejects; build with "
                   "--config=relax_clang_apple_flags to drop it (the output "
                   "may then differ from Xcode's)\n";
    }
    return 1;
  }

  if (!use_xcrun) {
    // XCTest bundles leave symbols for the test runner to resolve.
    if (ContainsArgument(processed_args, "-bundle") &&
        ContainsArgument(processed_args, "XCTest")) {
      processed_args.push_back("-Wl,-undefined,dynamic_lookup");
    }
  }

  // The layering check's `crosstool` module (APPLE_SUPPORT_MODULEMAP): the
  // toolchain writes it beside the developer directory, its headers' paths
  // relative to it, so it is read where it is, without apple_support's
  // overlay at $DEVELOPER_DIR/vfs.modulemap.
  const char *module_map_env = getenv("APPLE_SUPPORT_MODULEMAP");
  if (module_map_env) {
    processed_args.push_back("-Xclang");
    processed_args.push_back(std::string("-fmodule-map-file=") + module_map_env);
  }

  // Special mode that only prints the command. Used for testing.
  if (getenv("__WRAPPED_CLANG_LOG_ONLY")) {
    for (const std::string &arg : invocation_args) std::cout << arg << ' ';
    for (const std::string &arg : processed_args) std::cout << arg << ' ';
    std::cout << "\n";
    return 0;
  }

  auto response_file = WriteResponseFile(processed_args);
  invocation_args.push_back("@" + response_file->GetPath());

  // Check to see if we should postprocess with dsymutil.
  bool postprocess = !linked_binary.empty();

  if (!RunSubProcess(invocation_args)) {
    return 1;
  }
  if (!postprocess) {
    return 0;
  }

  // dsymutil and strip come from the same place clang did.
  auto tool = [&](const std::string &name, const std::string &llvm_name) {
    std::vector<std::string> args;
    if (use_xcrun) {
      args = {"/usr/bin/xcrun", name};
    } else {
      args = {toolchain_bin + "/" + llvm_name};
    }
    return args;
  };

  // Generate dSYM if dsym_path is set.
  if (!dsym_path.empty()) {
    std::vector<std::string> dsymutil_args = tool("dsymutil", "dsymutil");
    dsymutil_args.insert(dsymutil_args.end(),
                         {linked_binary, "-o", dsym_path, "--no-swiftmodule-timestamp"});
    // If dsym_path ends with .dSYM, generate a bundle; otherwise use --flat.
    if (dsym_path.size() < 5 ||
        dsym_path.compare(dsym_path.size() - 5, 5, ".dSYM") != 0) {
      dsymutil_args.push_back("--flat");
    }
    if (!RunSubProcess(dsymutil_args)) {
      return 1;
    }
  }

  // Strip debug symbols if requested.
  if (strip_debug_symbols) {
    std::vector<std::string> strip_args = tool("strip", "strip");
    strip_args.insert(strip_args.end(), {"-S", linked_binary});
    if (!RunSubProcess(strip_args)) {
      return 2;
    }
  }

  return 0;
}
