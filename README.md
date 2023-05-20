# LibRaw

**Not finished!**
The `libraw` crate provides a safe wrapper around the native `libraw` library.


## Dependencies
In order to use the `libraw` crate, you must have the `libraw_r` library installed where it can be
found by `pkg-config`. `libraw_r` is the reentrant version of LibRaw. Linking against the non
reentrant `libraw` is not supported.

On Debian-based Linux distributions, install the `libraw-dev` package:

```
sudo apt-get install libraw-dev
```

On OS X, install `libraw` with Homebrew:

```
brew install libraw
```

On FreeBSD, install the `libraw` package:

```
sudo pkg install libraw
```

## Usage
Add `libraw` as a dependency in `Cargo.toml`:

```toml
[dependencies]
libraw = "0.2"
```

Import the `libraw` crate. 

```rust

fn main() -> Result<(), Box<dyn Error>> {
    let filename = "test.CR2";
    libraw::init()?
        .load_image_from_path(filename)?
        .unpack()?
        .dcraw_process()?
        .dcraw_ppm_tiff_writer("test.tiff", true)?;
    Ok(())
}

```

## License
Copyright © 2023 DaLappen

Distributed under the [MIT License](LICENSE).

*Note:* By using this crate, your executable will link to the `libraw` C library, which is available
under the [LGPL version 2.1, CDDL version 1.0, or LibRaw Software
License](https://github.com/LibRaw/LibRaw/blob/master/COPYRIGHT).