use std::env;

fn main() {
    let paths = env::var_os("PATH").unwrap();
    let hyprctl_exists = env::split_paths(&paths)
        .map(|path| {
            let prog = path.join("hyprctl");
            prog.is_file()
        })
        .any(|path| path);
    if hyprctl_exists {
        println!("cargo::rustc-cfg=feature=\"hyprland\"");
    }
}
