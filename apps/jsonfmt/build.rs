fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("jsonfmt.ico"); // 图标文件路径
        res.compile().unwrap();
    }
}
