# Flashbang

Flashbang is an open-source flashcard application for Android, built as a personal alternative to Anki. It uses Rust for core logic and Jetpack Compose for the UI.

## Features

- **Typst-based Cards:** Renders rich text and mathematical equations using the Typst typesetting system.
- **FSRS Scheduler:** Implements the Free Spaced Repetition Scheduler (FSRS) algorithm.
- **GitHub Sync:** Loads and syncs card decks from GitHub repositories.
- **Fuzzy Search:** Finds specific cards using fuzzy search.
- **Statistics & Visualization:** Tracks progress with charts and statistics powered by Vico.
- **Material 3 Design:** Built with Jetpack Compose and Material 3 components.
- **Offline First:** Data is stored locally on the device.

## Architecture

Flashbang uses a hybrid architecture:

- **Frontend (Android):** Written in Kotlin using Jetpack Compose, Hilt, and Navigation 3.
- **Backend (Rust):** The core logic (scheduler, card parsing, search, GitHub sync) resides in the `fb-core` Rust crate.
- **Communication:** [UniFFI](https://github.com/mozilla/uniffi-rs) generates Kotlin bindings for the Rust library.

## Building

### Prerequisites

Ensure you have the following installed:

- **Rust Toolchain:** Install Rust via [rustup](https://rustup.rs/).
- **Android Studio:** With the Android SDK and NDK installed.
- **cargo-ndk:** Install with `cargo install cargo-ndk`.
- **uniffi-bindgen:** Install with `cargo install uniffi-bindgen`.

You will also need to add the Android targets to your Rust installation:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

### Steps

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/your-username/flashbang.git
    cd flashbang
    ```

2.  **Build the Rust core and generate bindings:**
    Run the build script from the `fb-core` directory:
    ```bash
    cd fb-core
    ./build-android.sh
    cd ..
    ```
    This script compiles the Rust code for Android architectures and generates the necessary Kotlin bindings in `app/src/main/java/dev/vndx/flashbang/rust`.

3.  **Run the Android App:**
    Open the project in Android Studio and run the `app` configuration on your device or emulator.
