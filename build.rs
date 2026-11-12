fn main() {
    slint_build::compile("ui/app.slint").expect("ui/app.slint 을 컴파일하지 못했다");

    // exe 아이콘과 버전 정보. 이게 없으면 작업 관리자·속성 창이 파일명을 앱 이름으로 쓴다.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set_language(0x0412);
        for key in ["CompanyName", "FileDescription", "ProductName", "InternalName"] {
            res.set(key, "QuickMedia");
        }
        res.set("OriginalFilename", "QuickMedia.exe");
        res.compile().expect("버전 리소스를 만들지 못했다");
    }
}
