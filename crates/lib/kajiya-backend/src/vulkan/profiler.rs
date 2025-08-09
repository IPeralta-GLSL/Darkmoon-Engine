#[cfg(feature = "gpu-profiler-enabled")]
use ash::vk;
#[cfg(feature = "gpu-profiler-enabled")]
use gpu_profiler::backend::ash::VulkanProfilerFrame;

#[cfg(feature = "gpu-profiler-enabled")]
pub struct ProfilerBuffer {
    buffer: vk::Buffer,
    allocation: gpu_allocator::SubAllocation,
}

#[cfg(feature = "gpu-profiler-enabled")]
pub struct ProfilerBackend<'dev, 'alloc> {
    device: &'dev ash::Device,
    allocator: &'alloc mut gpu_allocator::VulkanAllocator,
    timestamp_period: f32,
}

#[cfg(feature = "gpu-profiler-enabled")]
impl<'device, 'alloc> ProfilerBackend<'device, 'alloc> {
    pub fn new(
        device: &'device ash::Device,
        allocator: &'alloc mut gpu_allocator::VulkanAllocator,
        timestamp_period: f32,
    ) -> ProfilerBackend<'device, 'alloc> {
        Self {
            device,
            allocator,
            timestamp_period,
        }
    }
}

#[cfg(feature = "gpu-profiler-enabled")]
impl<'dev, 'alloc> gpu_profiler::backend::ash::VulkanBackend for ProfilerBackend<'dev, 'alloc> {
    type Buffer = ProfilerBuffer;

    fn create_query_result_buffer(&mut self, bytes: usize) -> Self::Buffer {
        let usage = vk::BufferUsageFlags::TRANSFER_DST;

        let buffer_info = vk::BufferCreateInfo {
            size: bytes as u64,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let buffer = unsafe {
            self.device
                .create_buffer(&buffer_info, None)
                .expect("create_buffer")
        };
        let requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };

        let allocation = self
            .allocator
            .allocate(&gpu_allocator::AllocationCreateDesc {
                name: "buffer",
                requirements,
                location: gpu_allocator::MemoryLocation::GpuToCpu,
                linear: true, // Buffers are always linear
            })
            .unwrap();

        // Bind memory to the buffer
        unsafe {
            self.device
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .expect("bind_buffer_memory")
        };

        ProfilerBuffer { buffer, allocation }
    }

    fn timestamp_period(&self) -> f32 {
        self.timestamp_period
    }
}

#[cfg(feature = "gpu-profiler-enabled")]
impl gpu_profiler::backend::ash::VulkanBuffer for ProfilerBuffer {
    fn mapped_slice(&self) -> &[u8] {
        self.allocation.mapped_slice().unwrap()
    }

    fn raw(&self) -> ash::vk::Buffer {
        self.buffer
    }
}

#[cfg(feature = "gpu-profiler-enabled")]
pub type VkProfilerData = VulkanProfilerFrame<ProfilerBuffer>;

#[cfg(not(feature = "gpu-profiler-enabled"))]
pub type VkProfilerData = ();

#[cfg(not(feature = "gpu-profiler-enabled"))]
pub struct ProfilerBuffer;

#[cfg(not(feature = "gpu-profiler-enabled"))]
pub struct ProfilerBackend<'dev, 'alloc> {
    _phantom: std::marker::PhantomData<(&'dev (), &'alloc ())>,
}

#[cfg(not(feature = "gpu-profiler-enabled"))]
impl<'device, 'alloc> ProfilerBackend<'device, 'alloc> {
    pub fn new(
        _device: &'device ash::Device,
        _allocator: &'alloc mut gpu_allocator::VulkanAllocator,
        _timestamp_period: f32,
    ) -> ProfilerBackend<'device, 'alloc> {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
