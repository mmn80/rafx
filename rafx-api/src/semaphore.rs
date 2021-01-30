#[cfg(any(
    feature = "rafx-empty",
    not(any(feature = "rafx-metal", feature = "rafx-vulkan"))
))]
use crate::empty::RafxSemaphoreEmpty;
#[cfg(feature = "rafx-metal")]
use crate::metal::RafxSemaphoreMetal;
#[cfg(feature = "rafx-vulkan")]
use crate::vulkan::RafxSemaphoreVulkan;

/// A GPU -> GPU synchronization mechanism.
///
/// A semaphore is either "signalled" or "unsignalled". Only the GPU can read or write this status.
///
/// Semaphores can be used to queue multiple dependent units of work to the GPU where one unit of
/// work cannot start until another unit of work completes.
///
/// Semaphores must not be dropped if they are in use by the GPU.
pub enum RafxSemaphore {
    #[cfg(feature = "rafx-vulkan")]
    Vk(RafxSemaphoreVulkan),
    #[cfg(feature = "rafx-metal")]
    Metal(RafxSemaphoreMetal),
    #[cfg(any(
        feature = "rafx-empty",
        not(any(feature = "rafx-metal", feature = "rafx-vulkan"))
    ))]
    Empty(RafxSemaphoreEmpty),
}

impl RafxSemaphore {
    /// Get the underlying vulkan API object. This provides access to any internally created
    /// vulkan objects.
    #[cfg(feature = "rafx-vulkan")]
    pub fn vk_semaphore(&self) -> Option<&RafxSemaphoreVulkan> {
        match self {
            #[cfg(feature = "rafx-vulkan")]
            RafxSemaphore::Vk(inner) => Some(inner),
            #[cfg(feature = "rafx-metal")]
            RafxSemaphore::Metal(_) => None,
            #[cfg(any(
                feature = "rafx-empty",
                not(any(feature = "rafx-metal", feature = "rafx-vulkan"))
            ))]
            RafxSemaphore::Empty(_) => None,
        }
    }

    /// Get the underlying metal API object. This provides access to any internally created
    /// metal objects.
    #[cfg(feature = "rafx-metal")]
    pub fn metal_semaphore(&self) -> Option<&RafxSemaphoreMetal> {
        match self {
            #[cfg(feature = "rafx-vulkan")]
            RafxSemaphore::Vk(_) => None,
            #[cfg(feature = "rafx-metal")]
            RafxSemaphore::Metal(inner) => Some(inner),
            #[cfg(any(
                feature = "rafx-empty",
                not(any(feature = "rafx-metal", feature = "rafx-vulkan"))
            ))]
            RafxSemaphore::Empty(inner) => None,
        }
    }

    /// Get the underlying metal API object. This provides access to any internally created
    /// metal objects.
    #[cfg(any(
        feature = "rafx-empty",
        not(any(feature = "rafx-metal", feature = "rafx-vulkan"))
    ))]
    pub fn empty_semaphore(&self) -> Option<&RafxSemaphoreEmpty> {
        match self {
            #[cfg(feature = "rafx-vulkan")]
            RafxSemaphore::Vk(_) => None,
            #[cfg(feature = "rafx-metal")]
            RafxSemaphore::Metal(_) => None,
            #[cfg(any(
                feature = "rafx-empty",
                not(any(feature = "rafx-metal", feature = "rafx-vulkan"))
            ))]
            RafxSemaphore::Empty(inner) => Some(inner),
        }
    }
}
