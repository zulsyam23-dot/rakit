#[link(name = "user32")]
unsafe extern "system" {
    pub fn EnableWindow(hWnd: isize, bEnable: i32) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn GetModuleHandleW(lpModuleName: *const u16) -> isize;
}
