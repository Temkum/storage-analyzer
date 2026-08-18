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
