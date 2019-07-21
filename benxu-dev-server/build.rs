use std::{ fs, path::Path, process::Command };

fn build_dirs(root: &Path) {
    let needed_dirs = [
        "wasm-pack",
        "wasm",
        "js",
        "js/wasm-bindgen-glue",
        "css",
        "img",
    ];
    for needed_dir in needed_dirs.iter() {
        if let Err(e) = fs::create_dir(root.join(needed_dir)) {
            println!("Directory not created! Error: {:?}", e);
        }
    }
}

fn move_wasm_and_loader(root: &Path) {
    for entry in fs::read_dir(root.join("wasm-pack")).unwrap() {
        let entry = entry.unwrap();
        let file = entry.file_name();
        if let Some(name) = file.to_str(){
            let path = entry.path();
            if name.ends_with(".js") {
                fs::copy(&path, root.join("js/wasm-bindgen-glue").join(name)).unwrap();
            }
            if name.ends_with(".wasm") {
                let wasm_file_path = root.join("wasm").join(name);
                fs::copy(&path, &wasm_file_path).unwrap();
                Command::new("wasm-gc")
                    .arg(&wasm_file_path)
                    .output()
                    .unwrap();
            }
        }
    }
}

fn main() {
    let static_file_dir = Path::new("./public");
    build_dirs(&static_file_dir);
    move_wasm_and_loader(&static_file_dir);
}

