# Protrustc

A protoc plugin for generating protrust code. This requires CMake to build. It's also recommended that you install vcpkg into the ignored third_party folder.

From the root repo directory you can then run:

```
> cmake -S protrustc -B protrustc\out -DCMAKE_TOOLCHAIN_FILE="{path-to-repo}\third_party\vcpkg\scripts\buildsystems\vcpkg.cmake"
```