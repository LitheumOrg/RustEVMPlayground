# litheum_wallet

This repository serves as a Mobile Wallet for Litheum Network.

## Getting Started

To begin, ensure that you have a working installation of the following items:
- [Flutter SDK](https://docs.flutter.dev/get-started/install)
- [Rust language](https://rustup.rs/)

### For Android:

- Appropriate [Rust targets](https://rust-lang.github.io/rustup/cross-compilation.html) are needed for cross-compiling to your device. 
  - For example:
  - ```bash
    rustup target add arm-linux-androidabi
    ```
  

- For Android targets you'll also need [cargo-ndk](https://github.com/bbqsrc/cargo-ndk#installing) ("handles all the environment configuration needed for successfully building libraries for Android from a Rust codebase.")

  - ```bash
    cargo install cargo-ndk
    ```
- For cross-compilation, a linker is also needed. Android NDS does the job:
  - ```bash
    brew install --cask android-ndk
    ```  
  - then add its path in one of the `gradle.properties`, e.g:
  - ```bash
    echo "ANDROID_NDK=.." >> ~/.gradle/gradle.properties
    ```

```
echo "ANDROID_NDK=.." >> ~/.gradle/gradle.properties
```

Then go ahead and run `flutter run`! When you're ready, refer to author's documentation
[here](https://fzyzcjy.github.io/flutter_rust_bridge/index.html)
THESE INSTRUCTIONS ARE NOW OUT OF DATE
to learn how to write and use binding code.

- On our linux platform, we can make use of the Android setup to test app on Android Simulator. Following [this guide](http://cjycode.com/flutter_rust_bridge/template/setup_android.html)
- Install codegen & `just` cmd helper (a modern command runner alternative to `Make`), following [this guide](http://cjycode.com/flutter_rust_bridge/template/generate_install.html).



Install 'just' (a modern command runner alternative to `Make`). e.g. on osx:
```
brew install just
```
Install ffigen ver 5.0.1 globally
```
dart pub global activate ffigen --version 5.0.1
```

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

just may ask you to run some other things. For example:

```
[2024-01-24T10:27:45Z INFO  lib_flutter_rust_bridge_codegen] Success!
[2024-01-24T10:27:45Z INFO  flutter_rust_bridge_codegen] Now go and use it :)
cp ios/Runner/bridge_generated.h macos/Runner/bridge_generated.h
# Uncomment this line to invoke build_runner as well
# flutter pub run build_runner build
cd native && cargo fmt
dart format .
```


## NOTES
- don't uncomment the package dependencies on our `native/Cargo.toml`, just replace by your local project dir instead.
At the moment, our repo is still private. If we use the git repo, it will lead to some permission errors when we run `flutter run`:
```
CMake Error at cmake_install.cmake:66 (file):
  file INSTALL cannot copy file
  "/home/thaodt/projects/LitheumOrg/LitheumMobileWallet/build/linux/x64/debug/intermediates_do_not_run/litheum_wallet"
  to "/usr/local/litheum_wallet": Permission denied.
```
i think will need to update `rust.cmake` in `linux/rust.cmake`, but lets do it later.