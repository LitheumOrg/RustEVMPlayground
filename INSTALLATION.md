# Installation Detail
1. All of us is using Linux, therefore I will point directly to [this instakllation link](https://docs.flutter.dev/get-started/install/linux). 
In the official docs, there are 2 choices, but i chose to install manually. 
After installed successfully, you will need to add `flutter` to your `PATH`.

2. Run flutter doctor
```
$ flutter doctor
```
- This command checks your environment and displays a report to the terminal window. The Dart SDK is bundled with Flutter; it is not necessary to install Dart separately. 
- Please notice the red marks, you may need to install or further tasks to perform (shown in bold text).

- You may need to install Android Studio to use Android simulator.
Just go to our https://aur.archlinux.org/packages/android-studio, clone & make build it.

- Next install [`cargo-ndk`](https://github.com/bbqsrc/cargo-ndk#installing), add rust target cross compilation.

- Finally for Android setup, you will need to [put its path](http://cjycode.com/flutter_rust_bridge/template/setup_android.html#android_ndk-gradle-property) in one of the `gradle.properties`, e.g.:
```
echo "ANDROID_NDK=(path to NDK)" >> ~/.gradle/gradle.properties
```
You may want to update that path in `LitheumFlutterDemo/android/gradle.properties`, im putting my path in that file.

- Rerun `flutter doctor` again to see all green checks.

- Install codegen & `just` cmd helper (a modern command runner alternative to `Make`), following [this guide](http://cjycode.com/flutter_rust_bridge/template/generate_install.html).


If your setup environment is ready, go to `How-to` section in `README` to run the app.

## Project Structure
This project should be available if you create a new Flutter project by command `flutter init`, but i made use of the template from author.
Just a bit different from the original flutter project initialization, you will see `native` dir. Its our Rust code, `api.rs` is pub ffi fns which we want to export for Dart code.
`flutter_rust_brigde` dependency package helped us generated `bridge_generated.rs` & `LitheumFlutterDemo/lib/bridge_generated.dart`.

So all `lib` files is DART code & `native` is our RUST code. 

As long as we don't touch GUI code on android/ios app, it's fine to use vscode or any editors/IDEs.

Another good news, Package's author is very active. 
He almost replied/supported within 10 mins, so if any queries, we can ping/create issue in his github repo.
