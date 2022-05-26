# Troubleshooting 
Refer to [this article](http://cjycode.com/flutter_rust_bridge/troubleshooting.html) first, you may find your problems & solutions there.

## We may find 3 first issues mentioned in the article above while setting up our dev environment (Android on your linux system).
1. http://cjycode.com/flutter_rust_bridge/troubleshooting.html#the-generated-store_dart_post_cobject-has-the-wrong-signature--stdargh-file-not-found-in-linux--stdboolh--

    This issue was reported by almost Arch users, so if you're an Arch fans, you may hit this.
    
    A workaround is running the generator command on `/` instead of normal directory.
By the time being, until this issue is resolved by [Dart ffigen](https://github.com/dart-lang/ffigen/issues/257), you may meet it.
2. http://cjycode.com/flutter_rust_bridge/troubleshooting.html#error-running-cargo-ndk-ld-error-unable-to-find-library--lgcc
    
    Solution was mentioned in the link, but I will bring it here also. This is an ongoing issue with `cargo-ndk`, a library unrelated to `flutter_rust_bridge` but solely used to build the examples, when using Android NDK version 23.

    A workaround is creating a file `libgcc.a` with this content `INPUT(-lunwind)` and put it in for each target system under your `cargo-ndk` path. For e.g.:
    ```
    YOUR_CARGO_NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/lib64/clang/14.0.1/lib/linux/x86_64/
    ```
    In my case, target systems are 4 dirs: `x86_64`, `aarch64`, `arm` & `i386`.

3. http://cjycode.com/flutter_rust_bridge/troubleshooting.html#issue-with-store_dart_post_cobject


## MacOS system
1. http://cjycode.com/flutter_rust_bridge/troubleshooting.html#fail-to-run-flutter_rust_bridge_codegen-on-macos-please-supply-one-or-more-pathtollvm

2. http://cjycode.com/flutter_rust_bridge/troubleshooting.html#on-m1-macos--failed-to-load-dynamic-library-opthomebrewoptllvmliblibclangdylib

## Development / Coding
1. http://cjycode.com/flutter_rust_bridge/troubleshooting.html#freezed-file-is-sometimes-not-generated-when-it-should-be
2. http://cjycode.com/flutter_rust_bridge/troubleshooting.html#cant-create-typedef-from-non-function-type
3. http://cjycode.com/flutter_rust_bridge/troubleshooting.html#generated-code-is-so-long
4. http://cjycode.com/flutter_rust_bridge/troubleshooting.html#why-need-dart-2140

## Other issues?

Don't hesitate to [open an issue](https://github.com/fzyzcjy/flutter_rust_bridge/issues/new/choose)! The author usually replies within minutes or hours (except when sleeping, of course).