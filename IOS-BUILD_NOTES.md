# NOTES
## Setup iOS Emulator on Linux
- Follow [this guide](https://dev.to/ianito/how-to-emulate-ios-on-linux-with-docker-4gj3), but remember chooosing macOS Monterey instead of Big Sur to install XCode >= v12.

    Just need to follow to steps before this section "Running a app with React Native". And the rest is just for reference.

## Install Rust & Setup Rust targets
- Install Rust - using [rustup](https://www.rust-lang.org/tools/install)
- http://cjycode.com/flutter_rust_bridge/template/setup_ios.html


## Install Flutter SDK
- Following this [official docs](https://docs.flutter.dev/get-started/install/macos).
- [iOS setup](https://docs.flutter.dev/get-started/install/macos#ios-setup)
- Setting up [iOS Simulator](https://docs.flutter.dev/get-started/install/macos#set-up-the-ios-simulator)

## Running app
- Following **How-to** section in [README.md](/README.md#how-to).


## Important notes
- This guide just helped you test app on macOS / iOS emulator. If you want to test on real device, you need to have real MAC device to plug-in your iPhone and signed certs.
- Some features related to push notification, camera/QRCode must be tested on real device, even build/release/test on **TestFlight**.
**TestFlight** required Apple Paid Developer Account (99$ / year). 
If we need it then we can subscribe to this package to do beta testing.