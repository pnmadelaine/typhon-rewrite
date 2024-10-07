fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    let output = std::path::PathBuf::from(args.next().unwrap());
    let input = std::path::PathBuf::from(args.next().unwrap());
    wasm_bindgen_cli_support::Bindgen::new()
        .input_path(input)
        .out_name("typhon")
        .web(true)
        .unwrap()
        .generate_output()
        .unwrap()
        .emit(output)
        .unwrap();
}
