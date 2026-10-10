//! Process shutdown gate regression (issue #620).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use lightcraft_develop::DevelopSettings;
use lightcraft_pipeline::{RenderRequest, SourceInfo};

#[test]
fn no_gpu_render_starts_after_process_shutdown() {
    let (width, height) = (600, 400);
    let source = Arc::new(lightcraft_scenes::demo_library()[0].render(width, height));
    let info = SourceInfo { raw: true, ..Default::default() };
    let settings = DevelopSettings::default();
    let request = RenderRequest::fit(width, height);
    let has_gpu = lightcraft_gpu::available();
    assert!(!lightcraft_gpu::shutting_down());
    assert!(lightcraft_gpu::wait_idle(Duration::ZERO));

    let rendered = Arc::new(AtomicUsize::new(0));
    let refused = Arc::new(AtomicBool::new(false));
    let worker = {
        let source = source.clone();
        let info = info.clone();
        let settings = settings.clone();
        let rendered = rendered.clone();
        let refused = refused.clone();
        std::thread::spawn(move || {
            while lightcraft_gpu::render(&source, &info, &settings, &request, None).is_some() {
                rendered.fetch_add(1, Ordering::SeqCst);
            }
            refused.store(true, Ordering::SeqCst);
        })
    };
    if has_gpu {
        while rendered.load(Ordering::SeqCst) == 0 && !refused.load(Ordering::SeqCst) {
            std::thread::yield_now();
        }
        assert!(rendered.load(Ordering::SeqCst) > 0);
    }

    lightcraft_gpu::begin_shutdown();
    assert!(lightcraft_gpu::shutting_down());
    assert!(lightcraft_gpu::wait_idle(Duration::from_secs(60)));
    worker.join().expect("render worker should stop");
    assert!(refused.load(Ordering::SeqCst));
    assert!(lightcraft_gpu::render(&source, &info, &settings, &request, None).is_none());
}
