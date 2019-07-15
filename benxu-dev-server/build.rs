use std::{ fs, path::Path, process::Command };

fn main() {
    let static_file_dir = Path::new("./public");
    let needed_dirs = [
        "wasm-pack",
        "wasm",
        "js",
        "js/wasm-bindgen-glue",
        "css",
        "img",
    ];
    for needed_dir in needed_dirs.iter() {
        fs::create_dir(static_file_dir.join(needed_dir)); // ignore
    }
    for entry in fs::read_dir(static_file_dir.join("wasm-pack")).unwrap() {
        let entry = entry.unwrap();
        let file = entry.file_name();
        if let Some(name) = file.to_str(){
            let path = entry.path();
            if name.ends_with(".js") {
                fs::copy(&path, static_file_dir.join("js/wasm-bindgen-glue").join(name)).unwrap();
            }
            if name.ends_with(".wasm") {
                let wasm_file_path = static_file_dir.join("wasm").join(name);
                fs::copy(&path, &wasm_file_path).unwrap();
                Command::new("wasm-gc")
                    .arg(&wasm_file_path)
                    .output()
                    .unwrap();
            }
        }
    }
}
