use std::error::Error;

pub(crate) fn set_immersive_mode() -> Result<(), Box<dyn Error>> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let env = vm.attach_current_thread()?;

    let window_obj = env.call_method(
        ctx.context() as jni::sys::jobject,
        "getWindow",
        "()Landroid/view/Window;",
        &[],
    )?;
    let decor_view_obj = env.call_method(
        window_obj.l()?,
        "getDecorView",
        "()Landroid/view/View;",
        &[],
    )?;

    let insets_controller = env.call_method(
        decor_view_obj.l()?,
        "getWindowInsetsController",
        "()Landroid/view/WindowInsetsController;",
        &[],
    )?;

    let flag = env.get_static_field(
        "android/view/WindowInsetsController",
        "BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE",
        "I",
    )?;

    env.call_method(
        insets_controller.l()?,
        "setSystemBarsBehavior",
        "(I)V",
        &[flag],
    )?;

    let flag = env.call_static_method("android/view/WindowInsets$Type", "systemBars", "()I", &[]);

    env.call_method(insets_controller.l()?, "hide", "(I)V", &[flag?])?;

    Ok(())
}
