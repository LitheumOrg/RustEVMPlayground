# litheum_wallet

This repository serves as a template for Flutter projects calling into native Rust
libraries via `flutter_rust_bridge`.

## Getting Started

To begin, ensure that you have a working installation of the following items:
- [Flutter SDK](https://docs.flutter.dev/get-started/install)
- [Rust language](https://rustup.rs/)
- Appropriate [Rust targets](https://rust-lang.github.io/rustup/cross-compilation.html) for cross-compiling to your device
- For Android targets:
    - Install [cargo-ndk](https://github.com/bbqsrc/cargo-ndk#installing)
    - Install Android NDK, then put its path in one of the `gradle.properties`, e.g.:

```
echo "ANDROID_NDK=.." >> ~/.gradle/gradle.properties
```

- Web is not supported yet.

Then go ahead and run `flutter run`! When you're ready, refer to author's documentation
[here](https://fzyzcjy.github.io/flutter_rust_bridge/index.html)
to learn how to write and use binding code.

- On our linux platform, we can make use of the Android setup to test app on Android Simulator. Following [this guide](http://cjycode.com/flutter_rust_bridge/template/setup_android.html)
- Install codegen & `just` cmd helper (a modern command runner alternative to `Make`), following [this guide](http://cjycode.com/flutter_rust_bridge/template/generate_install.html).
- If you want to play around with iOS/macOS setup, do [this](http://cjycode.com/flutter_rust_bridge/template/setup_ios.html) on your VM. 


## How-to
After you all set with dev environment, if anytime you made a change on rust code in `native` dir, remember to run the list of commands below so that it re-generates new build/generated bridge files for us:
```
$ flutter clean
$ dart pub get
$ flutter pub get
$ just
$ flutter run
```

## NOTES
- don't uncomment the package dependencies on our `native/Cargo.toml`, just replace by your local project dir instead.
At the moment, our repo is still private. If we use the git repo, it will lead to some permission errors when we run `flutter run`:
```
CMake Error at cmake_install.cmake:66 (file):
  file INSTALL cannot copy file
  "/home/thaodt/projects/LitheumOrg/LitheumFlutterDemo/build/linux/x64/debug/intermediates_do_not_run/litheum_wallet"
  to "/usr/local/litheum_wallet": Permission denied.
```
i think will need to update `rust.cmake` in `linux/rust.cmake`, but lets do it later.