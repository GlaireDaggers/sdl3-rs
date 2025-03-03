use crate::{
    get_error,
    gpu::{
        BufferBinding, BufferRegion, GraphicsPipeline, IndexElementSize, LoadOp, StoreOp, Texture,
        TextureRegion, TextureSamplerBinding, TextureTransferInfo, TransferBufferLocation,
    },
    pixels::Color,
    Error,
};
use sys::gpu::{
    SDL_AcquireGPUSwapchainTexture, SDL_BindGPUFragmentSamplers, SDL_BindGPUIndexBuffer,
    SDL_BindGPUVertexBuffers, SDL_DrawGPUIndexedPrimitives, SDL_GPUBlitInfo, SDL_GPUBufferBinding,
    SDL_GPUColorTargetInfo, SDL_GPUCommandBuffer, SDL_GPUComputePass, SDL_GPUCopyPass,
    SDL_GPUDepthStencilTargetInfo, SDL_GPUFilter, SDL_GPUIndexElementSize, SDL_GPULoadOp,
    SDL_GPURenderPass, SDL_GPUStoreOp, SDL_GPUTextureSamplerBinding, SDL_PushGPUComputeUniformData,
    SDL_PushGPUFragmentUniformData, SDL_PushGPUVertexUniformData, SDL_UploadToGPUBuffer,
    SDL_UploadToGPUTexture, SDL_WaitAndAcquireGPUSwapchainTexture,
};

use super::{Buffer, ComputePipeline, Filter};

pub struct CommandBuffer {
    pub(super) inner: *mut SDL_GPUCommandBuffer,
}
impl CommandBuffer {
    pub(super) fn new(inner: *mut SDL_GPUCommandBuffer) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn raw(&self) -> *mut SDL_GPUCommandBuffer {
        self.inner
    }

    #[doc(alias = "SDL_PushGPUVertexUniformData")]
    pub fn push_vertex_uniform_data<T: Sized>(&self, slot_index: u32, data: &T) {
        unsafe {
            SDL_PushGPUVertexUniformData(
                self.raw(),
                slot_index,
                (data as *const T) as *const std::ffi::c_void,
                size_of::<T>() as u32,
            )
        }
    }

    #[doc(alias = "SDL_PushGPUFragmentUniformData")]
    pub fn push_fragment_uniform_data<T: Sized>(&self, slot_index: u32, data: &T) {
        unsafe {
            SDL_PushGPUFragmentUniformData(
                self.raw(),
                slot_index,
                (data as *const T) as *const std::ffi::c_void,
                size_of::<T>() as u32,
            )
        }
    }

    #[doc(alias = "SDL_PushGPUComputeUniformData")]
    pub fn push_compute_uniform_data<T: Sized>(&self, slot_index: u32, data: &T) {
        unsafe {
            SDL_PushGPUComputeUniformData(
                self.raw(),
                slot_index,
                (data as *const T) as *const std::ffi::c_void,
                size_of::<T>() as u32,
            )
        }
    }

    #[doc(alias = "SDL_WaitAndAcquireGPUSwapchainTexture")]
    pub fn wait_and_acquire_swapchain_texture<'a>(
        &'a mut self,
        w: &crate::video::Window,
    ) -> Result<Texture<'a>, Error> {
        let mut swapchain = std::ptr::null_mut();
        let mut width = 0;
        let mut height = 0;
        let success = unsafe {
            SDL_WaitAndAcquireGPUSwapchainTexture(
                self.inner,
                w.raw(),
                &mut swapchain,
                &mut width,
                &mut height,
            )
        };
        if success {
            Ok(Texture::new_sdl_managed(swapchain, width, height))
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_AcquireGPUSwapchainTexture")]
    pub fn acquire_swapchain_texture<'a>(
        &'a mut self,
        w: &crate::video::Window,
    ) -> Result<Texture<'a>, Error> {
        let mut swapchain = std::ptr::null_mut();
        let mut width = 0;
        let mut height = 0;
        let success = unsafe {
            SDL_AcquireGPUSwapchainTexture(
                self.inner,
                w.raw(),
                &mut swapchain,
                &mut width,
                &mut height,
            )
        };
        if success {
            Ok(Texture::new_sdl_managed(swapchain, width, height))
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_BlitGPUTexture")]
    pub fn blit_texture<'a>(&'a self, blit_info: &BlitInfo) {
        unsafe {
            sys::gpu::SDL_BlitGPUTexture(self.inner, &blit_info.inner);
        }
    }

    #[doc(alias = "SDL_SubmitGPUCommandBuffer")]
    pub fn submit(self) -> Result<(), Error> {
        if unsafe { sys::gpu::SDL_SubmitGPUCommandBuffer(self.inner) } {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "SDL_CancelGPUCommandBuffer")]
    pub fn cancel(&mut self) {
        unsafe {
            sys::gpu::SDL_CancelGPUCommandBuffer(self.inner);
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct DepthStencilTargetInfo {
    inner: SDL_GPUDepthStencilTargetInfo,
}
impl DepthStencilTargetInfo {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_texture(mut self, texture: &mut Texture) -> Self {
        self.inner.texture = texture.raw();
        self
    }

    pub fn with_clear_depth(mut self, clear_depth: f32) -> Self {
        self.inner.clear_depth = clear_depth;
        self
    }

    pub fn with_load_op(mut self, value: LoadOp) -> Self {
        self.inner.load_op = SDL_GPULoadOp(value as i32);
        self
    }

    pub fn with_store_op(mut self, value: StoreOp) -> Self {
        self.inner.store_op = SDL_GPUStoreOp(value as i32);
        self
    }

    pub fn with_stencil_load_op(mut self, value: LoadOp) -> Self {
        self.inner.stencil_load_op =
            unsafe { std::mem::transmute::<_, sys::gpu::SDL_GPULoadOp>(value as u32) };
        self
    }

    pub fn with_stencil_store_op(mut self, value: StoreOp) -> Self {
        self.inner.stencil_store_op =
            unsafe { std::mem::transmute::<_, sys::gpu::SDL_GPUStoreOp>(value as u32) };
        self
    }

    pub fn with_cycle(mut self, cycle: bool) -> Self {
        self.inner.cycle = cycle;
        self
    }

    pub fn with_clear_stencil(mut self, clear_stencil: u8) -> Self {
        self.inner.clear_stencil = clear_stencil;
        self
    }
}

#[repr(C)]
#[derive(Default)]
pub struct ColorTargetInfo {
    inner: SDL_GPUColorTargetInfo,
}
impl ColorTargetInfo {
    pub fn with_texture(mut self, texture: &Texture) -> Self {
        self.inner.texture = texture.raw();
        self
    }
    pub fn with_load_op(mut self, value: LoadOp) -> Self {
        self.inner.load_op =
            unsafe { std::mem::transmute::<_, sys::gpu::SDL_GPULoadOp>(value as u32) };
        self
    }
    pub fn with_store_op(mut self, value: StoreOp) -> Self {
        self.inner.store_op =
            unsafe { std::mem::transmute::<_, sys::gpu::SDL_GPUStoreOp>(value as u32) };
        self
    }
    pub fn with_clear_color(mut self, value: Color) -> Self {
        self.inner.clear_color.r = (value.r as f32) / 255.0;
        self.inner.clear_color.g = (value.g as f32) / 255.0;
        self.inner.clear_color.b = (value.b as f32) / 255.0;
        self.inner.clear_color.a = (value.a as f32) / 255.0;
        self
    }
}

#[repr(C)]
#[derive(Default)]
pub struct BlitInfo {
    inner: SDL_GPUBlitInfo,
}
impl BlitInfo {
    pub fn with_source_texture(mut self, texture: &Texture) -> Self {
        self.inner.source.texture = texture.raw();
        self
    }

    pub fn with_source_mip(mut self, mip_level: u32) -> Self {
        self.inner.source.mip_level = mip_level;
        self
    }

    pub fn with_source_region(
        mut self,
        layer_or_depth: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Self {
        self.inner.source.layer_or_depth_plane = layer_or_depth;
        self.inner.source.x = x;
        self.inner.source.y = y;
        self.inner.source.w = width;
        self.inner.source.h = height;
        self
    }

    pub fn with_destination_texture(mut self, texture: &Texture) -> Self {
        self.inner.destination.texture = texture.raw();
        self
    }

    pub fn with_destination_mip(mut self, mip_level: u32) -> Self {
        self.inner.destination.mip_level = mip_level;
        self
    }

    pub fn with_destination_region(
        mut self,
        layer_or_depth: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Self {
        self.inner.destination.layer_or_depth_plane = layer_or_depth;
        self.inner.destination.x = x;
        self.inner.destination.y = y;
        self.inner.destination.w = width;
        self.inner.destination.h = height;
        self
    }

    pub fn with_load_op(mut self, load_op: LoadOp) -> Self {
        self.inner.load_op = SDL_GPULoadOp(load_op as i32);
        self
    }

    pub fn with_clear_color(mut self, clear_color: Color) -> Self {
        self.inner.clear_color.r = (clear_color.r as f32) / 255.0;
        self.inner.clear_color.g = (clear_color.g as f32) / 255.0;
        self.inner.clear_color.b = (clear_color.b as f32) / 255.0;
        self.inner.clear_color.a = (clear_color.a as f32) / 255.0;
        self
    }

    pub fn with_filter(mut self, filter: Filter) -> Self {
        self.inner.filter = SDL_GPUFilter(filter as i32);
        self
    }

    pub fn with_cycle(mut self, cycle: bool) -> Self {
        self.inner.cycle = cycle;
        self
    }
}

pub struct RenderPass {
    pub(super) inner: *mut SDL_GPURenderPass,
}
impl RenderPass {
    #[inline]
    pub fn raw(&self) -> *mut SDL_GPURenderPass {
        self.inner
    }

    #[doc(alias = "SDL_BindGPUGraphicsPipeline")]
    pub fn bind_graphics_pipeline(&self, pipeline: &GraphicsPipeline) {
        unsafe { sys::gpu::SDL_BindGPUGraphicsPipeline(self.inner, pipeline.raw()) }
    }

    #[doc(alias = "SDL_BindGPUVertexBuffer")]
    pub fn bind_vertex_buffers(&self, first_slot: u32, bindings: &[BufferBinding]) {
        unsafe {
            SDL_BindGPUVertexBuffers(
                self.raw(),
                first_slot,
                bindings.as_ptr() as *mut SDL_GPUBufferBinding,
                bindings.len() as u32,
            )
        }
    }

    #[doc(alias = "SDL_BindGPUIndexBuffer")]
    pub fn bind_index_buffer(&self, binding: &BufferBinding, index_element_size: IndexElementSize) {
        unsafe {
            SDL_BindGPUIndexBuffer(
                self.raw(),
                &binding.inner,
                SDL_GPUIndexElementSize(index_element_size as i32),
            )
        }
    }

    #[doc(alias = "SDL_BindGPUFragmentSamplers")]
    pub fn bind_fragment_sampler(&self, first_slot: u32, bindings: &[TextureSamplerBinding]) {
        unsafe {
            SDL_BindGPUFragmentSamplers(
                self.raw(),
                first_slot,
                bindings.as_ptr() as *const SDL_GPUTextureSamplerBinding,
                bindings.len() as u32,
            );
        }
    }

    #[doc(alias = "SDL_DrawGPUIndexedPrimitives")]
    pub fn draw_indexed_primitives(
        &self,
        num_indices: u32,
        num_instances: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            SDL_DrawGPUIndexedPrimitives(
                self.raw(),
                num_indices,
                num_instances,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }

    #[doc(alias = "SDL_DrawGPUPrimitives")]
    pub fn draw_primitives(
        &self,
        num_vertices: usize,
        num_instances: usize,
        first_vertex: usize,
        first_instance: usize,
    ) {
        unsafe {
            sys::gpu::SDL_DrawGPUPrimitives(
                self.inner,
                num_vertices as u32,
                num_instances as u32,
                first_vertex as u32,
                first_instance as u32,
            );
        }
    }
}

pub struct CopyPass {
    pub(super) inner: *mut SDL_GPUCopyPass,
}
impl CopyPass {
    #[inline]
    pub fn raw(&self) -> *mut SDL_GPUCopyPass {
        self.inner
    }

    #[doc(alias = "SDL_UploadToGPUBuffer")]
    pub fn upload_to_gpu_buffer(
        &self,
        transfer_buf_location: TransferBufferLocation,
        buffer_region: BufferRegion,
        cycle: bool,
    ) {
        unsafe {
            SDL_UploadToGPUBuffer(
                self.raw(),
                &transfer_buf_location.inner,
                &buffer_region.inner,
                cycle,
            )
        }
    }

    #[doc(alias = "SDL_UploadToGPUTexture")]
    pub fn upload_to_gpu_texture(
        &self,
        source: TextureTransferInfo,
        destination: TextureRegion,
        cycle: bool,
    ) {
        unsafe { SDL_UploadToGPUTexture(self.raw(), &source.inner, &destination.inner, cycle) }
    }
}

pub struct ComputePass {
    pub(super) inner: *mut SDL_GPUComputePass,
}
impl ComputePass {
    #[inline]
    pub fn raw(&self) -> *mut SDL_GPUComputePass {
        self.inner
    }

    #[doc(alias = "SDL_BindGPUComputePipeline")]
    pub fn bind_compute_pipeline(&self, pipeline: &ComputePipeline) {
        unsafe { sys::gpu::SDL_BindGPUComputePipeline(self.inner, pipeline.raw()) }
    }

    #[doc(alias = "SDL_BindGPUComputeStorageBuffers")]
    pub fn bind_compute_storage_buffers(&self, first_slot: u32, storage_buffers: &[&Buffer]) {
        let buffer_handles = storage_buffers.iter().map(|x| x.raw()).collect::<Vec<_>>();
        unsafe {
            sys::gpu::SDL_BindGPUComputeStorageBuffers(
                self.inner,
                first_slot,
                buffer_handles.as_ptr(),
                buffer_handles.len() as u32,
            )
        }
    }

    #[doc(alias = "SDL_BindGPUComputeStorageTextures")]
    pub fn bind_compute_storage_textures(&self, first_slot: u32, storage_textures: &[&Texture]) {
        let texture_handles = storage_textures.iter().map(|x| x.raw()).collect::<Vec<_>>();
        unsafe {
            sys::gpu::SDL_BindGPUComputeStorageTextures(
                self.inner,
                first_slot,
                texture_handles.as_ptr(),
                texture_handles.len() as u32,
            )
        }
    }

    #[doc(alias = "SDL_DispatchGPUCompute")]
    pub fn dispatch(&self, groupcount_x: u32, groupcount_y: u32, groupcount_z: u32) {
        unsafe {
            sys::gpu::SDL_DispatchGPUCompute(self.inner, groupcount_x, groupcount_y, groupcount_z)
        }
    }
}
