## YInjector

[![license](https://img.shields.io/badge/license-MIT-yellow.svg?style=flat)](https://github.com/piz-ewing/injector/blob/main/LICENSE)
![Language](https://img.shields.io/badge/language-rust-brightgreen)

- ✨ Fusion injector
- 👍 Easy to configure
- 🚅 Monitor base on ETW

## build

```bash
# windows-x86
cargo b --target=i686-pc-windows-msvc

# windows-x64
cargo b --target=x86_64-pc-windows-msvc

# for release
$env:RUSTFLAGS="--remap-path-prefix $HOME=~"
```

## config

```toml
[global]
mode = 'native' # 'native' 'yapi' 'wow64ext'
exit = false # exit after injection

[easy] # optional
'x86.exe' = 'dlls/x86.dll'

[[mix]]
name = 'x86.exe'
dll = 'dlls/x86.dll'
delay = 0            # optional

[mix.limit] # optional
module = 'user32.dll' # optional
gui = true            # optional

[[mix]]
name = 'x64.exe'
dll = 'dlls/x64.dll'

[mix.limit]
module = 'ws2_32.dll'
```

## run

```
./injector.exe [config_path]
```

![demo](./demo.png)

## todo

- ❌ [bug] YAPI and wow64ext is unstable, recommend using native mode
- Config Hot Reload

## ref

**_thx_**

[YAPI -- Yet Another Process Injector](https://github.com/ez8-co/yapi.git) @ez8-co

[rewolf-wow64ext](https://github.com/rwfpl/rewolf-wow64ext) @rwfpl

[pretty-env-logger](https://github.com/seanmonstar/pretty-env-logger.git) @seanmonstar

[remove absolute paths in release binary](https://users.rust-lang.org/t/how-to-remove-absolute-paths-in-release-binary/75969)

[windows-win-rs](https://github.com/DoumanAsh/windows-win-rs.git)
