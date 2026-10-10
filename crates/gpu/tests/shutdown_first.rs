//! Device creation must be blocked when process teardown starts before first use.

use std::sync::Arc;
use std::time::Duration;

use lightcraft_develop::DevelopSettings;
use lightcraft_pipeline::{RenderRequest, SourceInfo};

#[test]
fn shutdown_before_first_use_does_not_create_device() {
    lightcraft_gpu::begin_shutdown();
    lightcraft_gpu::warm_up();
    assert!(!lightcraft_gpu::available());
    assert_eq!(lightcraft_gpu::adapter_name(), None);
    let source = Arc::new(lightcraft_scenes::demo_library()[0].render(300, 200));
    let result = lightcraft_gpu::render(&source, &SourceInfo::default(), &DevelopSettings::default(), &RenderRequest::fit(300, 200), None);
    assert!(result.is_none());
    assert!(lightcraft_gpu::wait_idle(Duration::from_secs(60)));
    let switched_off = lightcraft_gpu::unavailable_reason().is_some_and(|reason| reason.starts_with("disabled by LIGHTCRAFT_GPU"));
    assert!(switched_off || !lightcraft_gpu::ready());
}
