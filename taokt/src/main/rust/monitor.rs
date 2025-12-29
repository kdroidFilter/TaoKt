use std::sync::Arc;

use crate::{PhysicalPositionI32, PhysicalSizeU32};

#[derive(uniffi::Object)]
pub struct Monitor {
    pub(crate) inner: tao::monitor::MonitorHandle,
}

#[uniffi::export]
impl Monitor {
    pub fn name(&self) -> Option<String> {
        self.inner.name()
    }

    pub fn size(&self) -> PhysicalSizeU32 {
        self.inner.size().into()
    }

    pub fn position(&self) -> PhysicalPositionI32 {
        self.inner.position().into()
    }

    pub fn scale_factor(&self) -> f64 {
        self.inner.scale_factor()
    }

    pub fn video_modes(&self) -> Vec<Arc<VideoMode>> {
        self.inner
            .video_modes()
            .map(|vm| Arc::new(VideoMode { inner: vm }))
            .collect()
    }

    pub fn debug_string(&self) -> String {
        format!("{:?}", self.inner)
    }
}

#[derive(uniffi::Object)]
pub struct VideoMode {
    pub(crate) inner: tao::monitor::VideoMode,
}

#[uniffi::export]
impl VideoMode {
    pub fn size(&self) -> PhysicalSizeU32 {
        self.inner.size().into()
    }

    pub fn bit_depth(&self) -> u16 {
        self.inner.bit_depth()
    }

    pub fn refresh_rate(&self) -> u16 {
        self.inner.refresh_rate()
    }

    pub fn monitor(&self) -> Arc<Monitor> {
        Arc::new(Monitor {
            inner: self.inner.monitor(),
        })
    }

    pub fn display_string(&self) -> String {
        format!("{}", self.inner)
    }

    pub fn debug_string(&self) -> String {
        format!("{:?}", self.inner)
    }
}

