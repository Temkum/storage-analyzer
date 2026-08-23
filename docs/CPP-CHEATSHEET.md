# C++ Cheatsheet

## Compilation and Build

### Configure CMake with Ninja

```bash
cmake -S . -B build -G Ninja
````

* `-S .` specifies the source directory.
* `-B build` specifies the build directory.
* `-G Ninja` tells CMake to generate Ninja build files.

### Build

```bash
cmake --build build
```

### Run

```bash
./build/system-analyzer
```

### Debug build

Our CMake configuration defaults to:

```text
CMAKE_BUILD_TYPE=Debug
```

Debug builds include debugging information and are suitable for GDB.

### Release build

```bash
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build build
```

### Clean rebuild

Ninja's build directory can get stale after moving/renaming files. When in
doubt, wipe it rather than debugging a phantom CMake cache issue:

```bash
rm -rf build
cmake -S . -B build -G Ninja
cmake --build build
```

### Verbose build (see the actual compiler invocations)

```bash
cmake --build build --verbose
```

### Parallel build

Ninja parallelizes by default, but you can cap or force worker count:

```bash
cmake --build build -j 4
```

### Build a single target

Useful when you're only touching one test binary and don't want to relink
everything:

```bash
cmake --build build --target scan-result-serializer-test
```

Available targets in this project: `system-analyzer`,
`directory-size-aggregator-test`, `scan-result-test`,
`scan-result-serializer-test`, `linux-file-scanner-test` (Linux only),
`linux-disk-usage-test` (Linux only), `linux-volume-provider-test` (Linux
only), `scan-schema-contract-test`.

### The project's own build script

`scripts/build-engine.sh` (`.ps1` on Windows) wraps configure + build + test

* sidecar staging in one step — prefer it over the raw CMake commands above
unless you're debugging the build itself:

```bash
./scripts/build-engine.sh

# Override generator or build type:
ENGINE_BUILD_TYPE=Release ./scripts/build-engine.sh

# Pass extra CMake args through:
./scripts/build-engine.sh -DCMAKE_EXPORT_COMPILE_COMMANDS=ON
```

### `compile_commands.json` for IDE/clangd support

```bash
cmake -S . -B build -G Ninja -DCMAKE_EXPORT_COMPILE_COMMANDS=ON
ln -sf build/compile_commands.json .
```

---

## Testing (ctest)

### Run the full suite

```bash
ctest --test-dir build --output-on-failure
```

`--output-on-failure` prints the failing test's stdout/stderr instead of
just "Failed" — always use it, the default is nearly useless for debugging.

### Run a single test by name

```bash
ctest --test-dir build -R scan-schema-contract
```

`-R` is a regex match against test names, so it also works for a group,
e.g. `-R linux-` runs every Linux-only platform test.

### List available tests without running them

```bash
ctest --test-dir build -N
```

### Run a test binary directly (bypasses ctest, useful under GDB)

```bash
./build/scan-result-serializer-test
```

### Rerun only what failed last time

```bash
ctest --test-dir build --rerun-failed --output-on-failure
```

---

## C++ Project Structure

Headers contain declarations/interfaces:

```cpp
class SystemInfo {
public:
    explicit SystemInfo(std::string name);
};
```

Implementation files contain definitions:

```cpp
SystemInfo::SystemInfo(std::string name)
    : name_(std::move(name)) {
}
```

Never include `.cpp` implementation files directly. Include headers and let the build system compile and link the translation units.

---

## Namespaces

We use:

```cpp
namespace system_analyzer::core {
    // ...
}
```

Namespaces prevent naming collisions and organize code.

---

## `const`

```cpp
const SystemInfo system("System Analyzer");
```

The object cannot be modified after initialization.

For member functions:

```cpp
const std::string& name() const;
```

The final `const` means the member function does not modify the object.

---

## References

```cpp
const std::string& name() const;
```

The return value is a reference to the existing string rather than a copy.

`const` prevents the caller from modifying the referenced value.

---

## `std::move`

```cpp
name_(std::move(name))
```

Allows resources owned by `name` to be transferred into `name_` instead of unnecessarily copying them.

`std::move` itself does not perform the move. It enables move semantics to be selected when the destination type supports them.

---

## `[[nodiscard]]`

```cpp
[[nodiscard]] const std::string& name() const;
```

Tells the compiler that callers should not silently ignore the return value.

---

## GDB

Start:

```bash
gdb ./build/system-analyzer
```

Disable Ubuntu debuginfod downloads when debugging locally:

```text
set debuginfod enabled off
```

Set breakpoint:

```text
break main
```

Start program:

```text
run
```

Execute the next source line:

```text
next
```

Inspect a variable:

```text
print variable
```

Continue execution:

```text
continue
```

Exit:

```text
quit
```

### Important debugging concept

If execution stops before an initialization statement:

```cpp
SystemInfo system("System Analyzer");
```

the object may not yet contain its intended initialized state.

Use `next` to execute the statement before inspecting it.

### Debug a test binary with an argument

```bash
gdb --args ./build/system-analyzer /home/user/some-directory
```

### Print a backtrace after a crash

```text
run
# ... crashes ...
bt
```

### Step *into* a function call vs. stepping over it

```text
step      # steps into the next function call
next      # steps over it
```

### Watch a variable (break when it changes)

```text
watch scannedEntries
continue
```

---

## Memory checking (Valgrind)

Not installed by default in CI, but useful locally when chasing a leak or
use-after-free in the scanner/aggregator:

```bash
sudo apt-get install valgrind
valgrind --leak-check=full ./build/system-analyzer /path/to/directory
```

---

## Platform-conditional CMake

The project picks its platform layer once, in `CMakeLists.txt`:

```cmake
if(WIN32)
    set(SYSTEM_ANALYZER_PLATFORM_DIR windows)
elseif(APPLE)
    set(SYSTEM_ANALYZER_PLATFORM_DIR macos)
else()
    set(SYSTEM_ANALYZER_PLATFORM_DIR linux)
endif()
```

When adding a new source file that's only valid on one OS (e.g. it includes
`<sys/statvfs.h>` or a Win32 header), guard the *target*, not just the
source list — CMake will still try to compile a listed source on every
platform unless the whole `add_executable`/`add_test` block is wrapped:

```cmake
if(NOT WIN32)
    add_executable(my-linux-only-test ...)
    add_test(NAME my-linux-only-test COMMAND my-linux-only-test)
endif()
```

This is exactly the fix that resolved the `sys/statvfs.h` Windows CI failure
in this project — the Linux-only test targets weren't gated, so MSVC tried
to build them.
