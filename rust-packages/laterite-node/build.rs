fn main() {
    // Emits the platform link args (macOS `-undefined dynamic_lookup`, etc.)
    // so the cdylib resolves Node's symbols at load time.
    napi_build::setup();
}
