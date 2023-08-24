## windows-injector


## build
```bash
# windows-x86
cargo b --target=i686-pc-windows-msvc

# windows-x64
cargo b --target=x86_64-pc-windows-msvc
```

## config
```toml
[global]
monitor_interval = 50

[injector]
"a.exe" = 'b.dll'
"b.exe" = '../c.dll'
"c.exe" = 'c:/1.dll'

```
## run

```
./injector.exe [config_path]
```

## todo
merge x86 and x64