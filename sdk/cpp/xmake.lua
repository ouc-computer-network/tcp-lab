-- xmake project for tcp-lab C++ SDK examples
-- 用于编译示例协议（如 TestSender），生成可被 Rust loader 加载的共享库。

set_project("tcp-lab-cpp-sdk")
set_version("0.1.0")

set_languages("cxx20")

-- 支持 debug/release 两种配置（默认由 xmake -m 控制）
add_rules("mode.debug", "mode.release")

-- 将本仓库的 include 目录加入头文件搜索路径
add_includedirs("include", { public = true })

-- 在 macOS/Linux 下链接 libtcp_lab_ffi.{dylib,so}，以便使用 tcp_lab_* 符号。
-- Windows 下需要按照实际路径调整 add_links/add_linkdirs。
-- os.scriptdir() -> <repo>/sdk/cpp
-- ../..          -> <repo> 根目录
local tcp_lab_root = path.join(os.scriptdir(), "..", "..")

-- 共享库目标：cpp_sender
target("cpp_sender")
    set_kind("shared")
    set_basename("cpp_sender")

    add_files("examples/TestSender.cpp")
    add_includedirs(tcp_lab_root .. "/crates/tcp-lab-ffi/src", { public = false })
    if is_plat("macosx") then
        add_shflags("-undefined dynamic_lookup", { force = true })
    end

-- RDT 3.0 Sender
target("rdt3_sender")
    set_kind("shared")
    set_basename("rdt3_sender")

    add_files("examples/Rdt3Sender.cpp")
    add_includedirs(tcp_lab_root .. "/crates/tcp-lab-ffi/src", { public = false })
    if is_plat("macosx") then
        add_shflags("-undefined dynamic_lookup", { force = true })
    end

-- RDT 3.0 Receiver
target("rdt3_receiver")
    set_kind("shared")
    set_basename("rdt3_receiver")

    add_files("examples/Rdt3Receiver.cpp")
    add_includedirs(tcp_lab_root .. "/crates/tcp-lab-ffi/src", { public = false })
    if is_plat("macosx") then
        add_shflags("-undefined dynamic_lookup", { force = true })
    end

    -- 在 Windows 上需要导出所有符号，xmake 会根据编译器自动处理；
    -- 如需自定义可在这里添加 add_cxflags / add_ldflags。


