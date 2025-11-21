use jni::{InitArgsBuilder, JavaVM};
use std::sync::Arc;
use tcp_lab_abstract::TransportProtocol;
use tcp_lab_jni::JavaTransportProtocol;

pub fn create_jvm(classpath: &str) -> anyhow::Result<Arc<JavaVM>> {
    // Detect library path (where libtcp_lab_jni.dylib/so is)
    // Assuming we run from cargo run, it is in target/debug/
    let lib_path = std::env::current_dir()?.join("target/debug");

    let jvm_args = InitArgsBuilder::new()
        .version(jni::JNIVersion::V8)
        .option(format!("-Djava.class.path={}", classpath))
        .option(format!("-Djava.library.path={}", lib_path.display()))
        .build()?;

    let jvm = JavaVM::new(jvm_args)?;
    {
        let mut env = jvm.attach_current_thread()?;
        tcp_lab_jni::register_native_methods(&mut env)?;
    }
    Ok(Arc::new(jvm))
}

pub fn load_protocol(
    jvm: &Arc<JavaVM>,
    class_name: &str,
) -> anyhow::Result<Box<dyn TransportProtocol>> {
    let mut env = jvm.attach_current_thread()?;

    let class_path = class_name.replace(".", "/");
    let cls = env.find_class(&class_path)?;
    let obj = env.new_object(cls, "()V", &[])?;
    let global = env.new_global_ref(obj)?;

    Ok(Box::new(JavaTransportProtocol::new(jvm.clone(), global)))
}
