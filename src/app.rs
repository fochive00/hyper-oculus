
use crate::config::*;
use crate::pipelines::Pipeline;
use crate::buffers::Buffer;
use crate::entities::{Vertex, Simplex as Entity};

use crate::cameras::Camera as CameraTrait;
use crate::cameras::{UniformBufferObject, CameraProj4 as Camera};

use nalgebra as na;
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;

// use egui_winit_ash_integration::Integration;
use ash::extensions::{
    ext::DebugUtils,
    khr::{Surface, Swapchain},
};

use ash::vk;
use std::ffi::CString;
use std::sync::{Arc, Mutex};
// use std::mem::ManuallyDrop;
// use std::ptr;
// use once_cell::unsync::OnceCell;
// use std::rc::Rc;

#[derive(Default)]
pub struct App {
    window: Option<Window>,
    entry: Option<ash::Entry>,
    instance: Option<ash::Instance>,
    max_frames_in_flight: Option<usize>,

    debug_utils_loader: Option<DebugUtils>,
    debug_utils_messenger: Option<vk::DebugUtilsMessengerEXT>,

    surface: Option<vk::SurfaceKHR>,
    surface_loader: Option<Surface>,

    physical_device: Option<vk::PhysicalDevice>,
    queue_family_index: Option<u32>,
    device: Option<ash::Device>,
    allocator: Option<Arc<Mutex<Allocator>>>,

    surface_format: Option<vk::SurfaceFormatKHR>,
    surface_resolution: Option<vk::Extent2D>,
    swapchain: Option<vk::SwapchainKHR>,
    swapchain_loader: Option<Swapchain>,
    swapchain_images: Option<Vec<vk::Image>>,
    swapchain_image_views: Option<Vec<vk::ImageView>>,
    swapchain_image_count: Option<usize>,

    command_pool: Option<vk::CommandPool>,
    setup_command_buffer: Option<vk::CommandBuffer>,
    draw_command_buffers: Option<Vec<vk::CommandBuffer>>,

    image_available_semaphores: Option<Vec<vk::Semaphore>>,
    render_finished_semaphores: Option<Vec<vk::Semaphore>>,
    inflight_fences: Option<Vec<vk::Fence>>,
    images_inflight: Option<Vec<vk::Fence>>,
    current_frame: Option<usize>,

    depth_image: Option<vk::Image>,
    depth_image_allocation: Option<Allocation>,
    depth_image_view: Option<vk::ImageView>,

    present_queue: Option<vk::Queue>,
    
    framebuffers: Option<Vec<vk::Framebuffer>>,

    descriptor_pool: Option<vk::DescriptorPool>,
    descriptor_set_layouts: Option<Vec<vk::DescriptorSetLayout>>,
    descriptor_sets: Option<Vec<vk::DescriptorSet>>,
    
    pipeline: Option<Pipeline>,

    entities: Option<Vec<Entity>>,
    camera: Option<Camera>,

    uniform_buffers: Option<Vec<Buffer>>,
    vertex_buffers: Option<Vec<Buffer>>,
    index_buffers: Option<Vec<Buffer>>,

    // egui_integration: Option<Integration<Arc<Mutex<Allocator>>>>,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let mut app = Self::default();

        app.max_frames_in_flight = Some(2);
        app.current_frame = Some(0);

        app.surface_resolution = Some(
            vk::Extent2D {
                width: WINDOW_WIDTH,
                height: WINDOW_WIDTH,
            }
        );

        app.create_window(event_loop);
        app.init_entry();
        app.create_instance();
        app.init_debug_utils_loader();
        app.create_surface();
        app.pick_physical_device();
        app.create_logical_device();

        app.create_allocator();

        app.create_swapchain();
        app.create_image_views();

        app.create_command_pool();
        app.create_setup_command_buffer();
        app.create_graphic_queue();

        app.create_depth_resource();

        app.create_descriptor_set_layouts();
        app.create_pipeline();

        // app.create_egui_integration();

        app.create_entities();
        app.create_camera();
        app.create_vertex_buffers();
        app.create_index_buffer();
        app.create_uniform_buffers();

        app.create_descriptor_pool();
        app.create_descriptor_sets();

        app.create_framebuffers();

        app.create_draw_command_buffers();
        app.create_sync_objects();

        app
    }

    fn create_window(&mut self, event_loop: &EventLoop<()>) {

        let window = WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build(&event_loop)
            .expect("Failed to create window.");
        
            // Grab and hide the cursor
        window.set_cursor_visible(false);
        window.set_cursor_grab(true)
            .expect("Failed to grab the cursor.");

        self.window = Some(window);
    }
    
    fn init_entry(&mut self) {
        let entry = ash::Entry::linked();

        self.entry = Some(entry);
    }

    fn create_instance(&mut self) {
        let entry = self.entry.as_ref()
            .expect("Could not get `entry`.");
        let window = self.window.as_ref()
            .expect("Could not get `window`.");

        let instance = unsafe {
            let app_name = CString::new(APP_NAME).unwrap();
            let appinfo = vk::ApplicationInfo::builder()
                .application_name(&app_name)
                .application_version(0)
                .api_version(vk::make_api_version(0, 1, 0, 0));

            let layer_names = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
            let layers_names_raw: Vec<*const i8> = layer_names
                .iter()
                .map(|raw_name| raw_name.as_ptr())
                .collect();
    
            let surface_extensions = ash_window::enumerate_required_extensions(window).unwrap();
            let extension_names_raw = {
                let mut extension_names_raw = surface_extensions
                    .iter()
                    .map(|ext| ext.as_ptr())
                    .collect::<Vec<_>>();
                extension_names_raw.push(DebugUtils::name().as_ptr());

                extension_names_raw
            };

            let create_info = vk::InstanceCreateInfo::builder()
                .application_info(&appinfo)
                .enabled_layer_names(&layers_names_raw)
                .enabled_extension_names(&extension_names_raw);

            entry
                .create_instance(&create_info, None)
                .expect("Failed to create `instance`")
        };

        self.instance = Some(instance);
    }

    fn init_debug_utils_loader(&mut self) {
        use crate::helpers::vulkan_debug_callback;

        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));

        let entry = self.entry.as_ref()
            .expect("Could not get `entry`");
        
        let instance = self.instance.as_ref()
            .expect("Could not get `instance`");

        let debug_utils_loader = DebugUtils::new(entry, instance);
        let debug_utils_messenger = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap()
        };

        self.debug_utils_loader = Some(debug_utils_loader);
        self.debug_utils_messenger = Some(debug_utils_messenger);
    }

    fn create_surface(&mut self) {
        let entry = self.entry.as_ref()
            .expect("Could not get `entry`.");
        let instance = self.instance.as_ref()
            .expect("Could not get `instance`.");
        let window = self.window.as_ref()
            .expect("Could not get `window`.");

        let surface = unsafe {
            ash_window::create_surface(entry, instance, window, None).unwrap()
        };

        let surface_loader = Surface::new(entry, instance);

        self.surface = Some(surface);
        self.surface_loader = Some(surface_loader);
    }
    
    fn pick_physical_device(&mut self) {

        let instance = self.instance.as_ref()
            .expect("Could not get instance");

        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Physical device error")
        };

        let surface = self.surface.as_ref()
            .expect("Could not get `surface`");
        let surface_loader = self.surface_loader.as_ref()
            .expect("Could not get `surface_loader`.");

        let (physical_device, queue_family_index) = unsafe {
            physical_devices
                .iter()
                .map(|physical_device| {
                    instance
                        .get_physical_device_queue_family_properties(*physical_device)
                        .iter()
                        .enumerate()
                        .find_map(|(index, info)| {
                            let supports_graphic_and_surface =
                                info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                    && surface_loader
                                        .get_physical_device_surface_support(
                                            *physical_device,
                                            index as u32,
                                            *surface,
                                        )
                                        .unwrap();
                            if supports_graphic_and_surface {
                                Some((*physical_device, index))
                            } else {
                                None
                            }
                        })
                })
                .flatten()
                .next()
                .expect("Couldn't find suitable device.")
        };

        self.physical_device = Some(physical_device);
        self.queue_family_index = Some(queue_family_index as u32);
    }

    fn create_logical_device(&mut self) {
        let device_extension_names_raw = [Swapchain::name().as_ptr()];
        let features = vk::PhysicalDeviceFeatures {
            shader_clip_distance: 1,
            ..Default::default()
        };
        let priorities = [1.0];

        let queue_family_index = self.queue_family_index.unwrap();

        let queue_info = [vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities)
            .build()];

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_info)
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features);

        let instance = self.instance.as_ref()
            .expect("Could not get instance");
        let physical_device = self.physical_device.as_ref()
            .expect("Could not get physical_device");

        let device = unsafe {
            instance
                .create_device(*physical_device, &device_create_info, None)
                .unwrap()
        };

        self.device = Some(device);
    }

    fn create_swapchain(&mut self) {
        let physical_device = self.physical_device.as_ref()
            .expect("Could not get `physical_device`.");
        let surface = self.surface.as_ref()
            .expect("Could not get `surface`.");
        let surface_loader = self.surface_loader.as_ref()
            .expect("Could not get `surface_loader`.");

        let surface_capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(*physical_device, *surface)
                .unwrap()
        };

        let surface_resolution = match surface_capabilities.current_extent.width {
            std::u32::MAX => self.surface_resolution.unwrap(),
            _ => surface_capabilities.current_extent,
        };

        let surface_format = unsafe {
            surface_loader
                .get_physical_device_surface_formats(*physical_device, *surface)
                .unwrap()[0]
        };

        let mut desired_image_count = surface_capabilities.min_image_count + 1;

        if surface_capabilities.max_image_count > 0 && desired_image_count > surface_capabilities.max_image_count {
            desired_image_count = surface_capabilities.max_image_count;
        }

        let pre_transform = if surface_capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY) {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_capabilities.current_transform
        };

        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(*physical_device, *surface)
                .unwrap()
        };

        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);
        
        let instance = self.instance.as_ref()
            .expect("Could not get `instance`.");
        let device = self.device.as_ref()
            .expect("Could not get `device`.");

        let swapchain_loader = Swapchain::new(instance, device);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(*surface)
            .min_image_count(desired_image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(surface_resolution)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1);

        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap()
        };

        self.surface_format = Some(surface_format);
        self.surface_resolution = Some(surface_resolution);
        self.swapchain_loader = Some(swapchain_loader);
        self.swapchain = Some(swapchain);
    }

    fn create_image_views(&mut self) {
        let swapchain_loader = self.swapchain_loader.as_ref()
            .expect("Could not get `swapchain_loader`.");
        
        let swapchain = self.swapchain.as_ref()
            .expect("Could not get `swapchain`.");

        let swapchain_images = unsafe {
            swapchain_loader.get_swapchain_images(*swapchain).unwrap()
        };

        let swapchain_image_count = swapchain_images.len();

        let surface_format = self.surface_format.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();

        let swapchain_image_views: Vec<vk::ImageView> = unsafe {
            swapchain_images
                .iter()
                .map(|&image| {
                    let create_view_info = vk::ImageViewCreateInfo::builder()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(surface_format.format)
                        .components(vk::ComponentMapping {
                            r: vk::ComponentSwizzle::R,
                            g: vk::ComponentSwizzle::G,
                            b: vk::ComponentSwizzle::B,
                            a: vk::ComponentSwizzle::A,
                        })
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        })
                        .image(image);
                    device.create_image_view(&create_view_info, None).unwrap()
                })
                .collect()
        };

        self.swapchain_images = Some(swapchain_images);
        self.swapchain_image_views = Some(swapchain_image_views);
        self.swapchain_image_count = Some(swapchain_image_count);
    }
    
    fn create_command_pool(&mut self) {
        let queue_family_index = self.queue_family_index.unwrap();

        let device = self.device.as_ref()
            .expect("Could not get `device`.");

        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);

        let pool = unsafe {
            device.create_command_pool(&pool_create_info, None).unwrap()
        };

        self.command_pool = Some(pool);
    }
    
    fn single_time_command<F: FnOnce(&ash::Device, vk::CommandBuffer)>(&self, f: F) {
        let device = self.device.as_ref().unwrap();
        let command_buffer = self.setup_command_buffer.as_ref().unwrap();
        let queue = self.present_queue.as_ref().unwrap();

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            device.begin_command_buffer(*command_buffer, &command_buffer_begin_info)
                .expect("Begin commandbuffer");
        }

        f(device, *command_buffer);

        unsafe {
            device.end_command_buffer(*command_buffer)
                .expect("End commandbuffer");
        }

        let command_buffers = &[*command_buffer];

        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(command_buffers);

        unsafe {
            device
                .queue_submit(
                    *queue,
                    &[submit_info.build()],
                    vk::Fence::null(),
                )
                .expect("Failed to submit draw command.");
            
            device.device_wait_idle().unwrap();
        }

    }

    fn create_depth_resource(&mut self) {
        
        // fn find_supported_format(candidates: &Vec<vk::Format>, tilling: vk::ImageTiling, features: vk::FormatFeatureFlags) {
        //     for format in candidates {
        //         let props = 
        //     }
        // }
        
        let device = self.device.as_ref().unwrap();
        let surface_resolution = self.surface_resolution.as_ref().unwrap();
        let allocator = self.allocator.as_ref().unwrap();

        let depth_image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::D32_SFLOAT)
            .extent(
                vk::Extent3D {
                    width: surface_resolution.width,
                    height: surface_resolution.height,
                    depth: 1,
                })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        
        let depth_image = unsafe {
            device.create_image(&depth_image_create_info, None).unwrap()
        };

        let depth_image_memory_req = unsafe {
            device.get_image_memory_requirements(depth_image)
        };

        let allocation = allocator
            .lock().unwrap()
            .allocate(&AllocationCreateDesc {
                name: "buffer memory allocation",
                requirements: depth_image_memory_req,
                location: MemoryLocation::GpuOnly,
                linear: true, // Buffers are always linear
            }).unwrap();

        unsafe {
            device
                .bind_image_memory(depth_image, allocation.memory(), allocation.offset())
                .expect("Unable to bind depth image memory");
        }

        self.single_time_command(|device, command_buffer| {
            unsafe {
                let layout_transition_barriers = vk::ImageMemoryBarrier::builder()
                    .image(depth_image)
                    .dst_access_mask(
                        vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                            | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                    )
                    .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .subresource_range(
                        vk::ImageSubresourceRange::builder()
                            .aspect_mask(vk::ImageAspectFlags::DEPTH)
                            .layer_count(1)
                            .level_count(1)
                            .build(),
                    );

                device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[layout_transition_barriers.build()],
                    );
            }
        });

        let depth_image_view_info = vk::ImageViewCreateInfo::builder()
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH)
                    .level_count(1)
                    .layer_count(1)
                    .build(),
            )
            .image(depth_image)
            .format(depth_image_create_info.format)
            .view_type(vk::ImageViewType::TYPE_2D);

        let depth_image_view = unsafe {
            device
                .create_image_view(&depth_image_view_info, None)
                .unwrap()
        };

        self.depth_image = Some(depth_image);
        self.depth_image_allocation = Some(allocation);
        self.depth_image_view = Some(depth_image_view);
    }

    fn create_setup_command_buffer(&mut self) {
        let device = self.device.as_ref().unwrap();
        let command_pool = self.command_pool.as_ref().unwrap();

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(*command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let setup_command_buffers= unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Could not allocate command buffers.")
        };

        self.setup_command_buffer = Some(setup_command_buffers[0]);
    }

    fn create_draw_command_buffers(&mut self) {
        let device = self.device.as_ref().unwrap();
        let command_pool = self.command_pool.as_ref().unwrap();
        let swapchain_image_count = self.swapchain_image_count.unwrap();

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(swapchain_image_count as u32)
            .command_pool(*command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let draw_command_buffers= unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Could not allocate command buffers.")
        };

        // record draw commands
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let descriptor_sets = self.descriptor_sets.as_ref()
            .expect("Could not get `descriptor_sets`.");
        let pipeline = self.pipeline.as_ref()
            .expect("Could not get `pipeline`.");

        let graphics_pipeline = pipeline.pipeline();
        let render_pass = pipeline.render_pass();
        let pipeline_layout = pipeline.pipeline_layout();
        
        let framebuffers = self.framebuffers.as_ref()
            .expect("Could not get `framebuffers`.");
        let surface_resolution = self.surface_resolution.as_ref()
            .expect("Could not get `surface_resolution`.");

        let device = self.device.as_ref()
            .expect("Could not get `device`.");

        let vertex_buffers = self.vertex_buffers.as_ref()
            .expect("Could not get `vertex_buffers`");
        let index_buffers = self.index_buffers.as_ref()
            .expect("Could not get `index_buffers`.");
        let entities = self.entities.as_ref()
            .expect("Could not get `entity`.");
        let index_count = entities[0].indices().len() as u32;

        for i in 0..swapchain_image_count {
            let command_buffer = draw_command_buffers[i];

            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(*render_pass)
                .framebuffer(framebuffers[i])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: *surface_resolution,
                })
                .clear_values(&clear_values);

            // recreate viewports and scissors
            // TODO remove these code
            let viewports = [vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: surface_resolution.width as f32,
                height: surface_resolution.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }];
    
            let scissors = [vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: *surface_resolution,
            }];

            unsafe {
                device
                    .reset_command_buffer(
                        command_buffer,
                        vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                    )
                    .expect("Reset command buffer failed.");
        
                let command_buffer_begin_info = vk::CommandBufferBeginInfo::default();
        
                device
                    .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                    .expect("Begin commandbuffer");
                    
                device.cmd_begin_render_pass(
                    command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );
                device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    *graphics_pipeline,
                );
    
                device.cmd_set_viewport(command_buffer, 0, &viewports);
                device.cmd_set_scissor(command_buffer, 0, &scissors);
                device.cmd_bind_vertex_buffers(
                    command_buffer,
                    0,
                    &[*vertex_buffers[0].buffer()],
                    &[0],
                );
                device.cmd_bind_index_buffer(
                    command_buffer,
                    *index_buffers[0].buffer(),
                    0,
                    vk::IndexType::UINT16,
                );
    
                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    *pipeline_layout,
                    0,
                    &[descriptor_sets[i]],
                    &[],
                );
    
                device.cmd_draw_indexed(
                    command_buffer,
                    index_count,
                    1,
                    0,
                    0,
                    1,
                );

                device.cmd_end_render_pass(command_buffer);

                // self.egui_ui(command_buffer, i);
                /////////////////////////////////////// egui /////////////////////////////////
                // let egui_integration = self.egui_integration.as_mut().unwrap();
                // let window = self.window.as_ref().unwrap();

                // egui_integration.begin_frame();
                // egui::SidePanel::left("my_side_panel").show(&egui_integration.context(), |ui| {
                //     ui.heading("Hello");
                //     ui.label("Hello egui!");
                //     ui.separator();
                //     ui.hyperlink("https://github.com/emilk/egui");
                //     ui.separator();
                //     ui.label("Rotation");
                // });
                // let (_, shapes) = egui_integration.end_frame(&window);

                // let clipped_meshes = egui_integration.context().tessellate(shapes);
                // egui_integration
                //     .paint(command_buffer, i, clipped_meshes);
                /////////////////////////////////////// egui /////////////////////////////////

                device
                    .end_command_buffer(command_buffer)
                    .expect("End commandbuffer");
            }
        }

        self.draw_command_buffers = Some(draw_command_buffers);
    }

    fn create_sync_objects(&mut self) {
        let device = self.device.as_ref().unwrap();
        let swapchain_image_count = self.swapchain_image_count.unwrap();
        let max_frames_in_flight = self.max_frames_in_flight.unwrap();

        let images_inflight = vec![vk::Fence::null(); swapchain_image_count];

        let mut image_available_semaphores = Vec::with_capacity(max_frames_in_flight);
        let mut render_finished_semaphores = Vec::with_capacity(max_frames_in_flight);
        let mut inflight_fences = Vec::with_capacity(max_frames_in_flight);

        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);

        let semaphore_create_info = vk::SemaphoreCreateInfo::default();

        for _ in 0..max_frames_in_flight {
            unsafe {
                let image_available_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .unwrap();

                let render_finished_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .unwrap();

                let inflight_fence = device
                        .create_fence(&fence_create_info, None)
                        .expect("Create fence failed.");

                image_available_semaphores.push(image_available_semaphore);
                render_finished_semaphores.push(render_finished_semaphore);
                inflight_fences.push(inflight_fence);
            }
        }

        self.inflight_fences = Some(inflight_fences);
        self.images_inflight = Some(images_inflight);

        self.image_available_semaphores = Some(image_available_semaphores);
        self.render_finished_semaphores = Some(render_finished_semaphores);
    }

    fn create_graphic_queue(&mut self) {
        let device = self.device.as_ref().unwrap();
        let queue_family_index = self.queue_family_index.unwrap();

        let present_queue = unsafe {
            device.get_device_queue(queue_family_index as u32, 0)
        };

        self.present_queue = Some(present_queue);
    }

    fn create_descriptor_pool(&mut self) {
        let device = self.device.as_ref().unwrap();
        let swapchain_image_count = self.swapchain_image_count.unwrap() as u32;

        let descriptor_pool_size = vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(swapchain_image_count);

        let pool_sizes = &[descriptor_pool_size.build()];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(pool_sizes)
            .max_sets(swapchain_image_count);

        let descriptor_pool = unsafe {
            device.create_descriptor_pool(&descriptor_pool_create_info, None).unwrap()
        };


        self.descriptor_pool = Some(descriptor_pool);
    }

    fn create_descriptor_set_layouts(&mut self) {

        let device = self.device.as_ref().unwrap();
        let swapchain_image_count = self.swapchain_image_count.unwrap();

        let ubo_layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        let layout_bindings = &[ubo_layout_binding.build()];
        let layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(layout_bindings);

        let mut descriptor_set_layouts = Vec::with_capacity(swapchain_image_count);


        for _ in 0..swapchain_image_count {
            let descriptor_set_layout = unsafe {
                device.create_descriptor_set_layout(&layout_create_info, None).unwrap()
            };
            
            descriptor_set_layouts.push(descriptor_set_layout);
        }

        self.descriptor_set_layouts = Some(descriptor_set_layouts);
    }

    fn create_descriptor_sets(&mut self) {
        let device = self.device.as_ref().unwrap();
        let swapchain_image_count = self.swapchain_image_count.unwrap();
        let uniform_buffers = self.uniform_buffers.as_ref().unwrap();
        let descriptor_pool = self.descriptor_pool.as_ref().unwrap();
        let set_layouts = self.descriptor_set_layouts.as_ref().unwrap();

        let allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(*descriptor_pool)
            .set_layouts(set_layouts.as_slice());

        let descriptor_sets = unsafe {
            device.allocate_descriptor_sets(&allocate_info).unwrap()
        };

        for i in 0..swapchain_image_count {
            let descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
                .offset(0)
                .buffer(*uniform_buffers[i].buffer())
                .range(std::mem::size_of::<UniformBufferObject>() as u64);

            let buffer_infos = &[descriptor_buffer_info.build()];

            let descriptor_writes = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(buffer_infos);

            unsafe {
                device.update_descriptor_sets(&[descriptor_writes.build()], &[]);
            }

        }

        self. descriptor_sets = Some(descriptor_sets);

    }

    fn create_pipeline(&mut self) {
        let device = self.device.as_ref().unwrap();
        let surface_format = self.surface_format.as_ref().unwrap();
        let surface_resolution = self.surface_resolution.as_ref().unwrap();
        let descriptor_set_layout = &self.descriptor_set_layouts.as_ref().unwrap()[0];

        let pipeline = Pipeline::new(device.clone(), surface_format, surface_resolution, descriptor_set_layout);

        self.pipeline = Some(pipeline);
    }

    fn create_framebuffers(&mut self) {

        let device = self.device.as_ref().unwrap();
        let swapchain_image_views = self.swapchain_image_views.as_ref().unwrap();
        let depth_image_view = self.depth_image_view.as_ref().unwrap();
        let surface_resolution = self.surface_resolution.as_ref().unwrap();
        let render_pass = self.pipeline.as_ref().unwrap().render_pass();

        let framebuffers: Vec<vk::Framebuffer> = swapchain_image_views
            .iter()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view, *depth_image_view];
                // let framebuffer_attachments = [present_image_view];
                let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(*render_pass)
                    .attachments(&framebuffer_attachments)
                    .width(surface_resolution.width)
                    .height(surface_resolution.height)
                    .layers(1);
                unsafe {
                    device
                        .create_framebuffer(&framebuffer_create_info, None)
                        .unwrap()
                }
            })
            .collect();

        self.framebuffers = Some(framebuffers);
    }

    fn create_entities(&mut self) {
        let entities = vec![Entity::new()];

        self.entities = Some(entities);
    }

    fn create_camera(&mut self) {
        let camera = Camera::new();

        self.camera = Some(camera);
    }

    fn create_vertex_buffers(&mut self) {
        let device = self.device.as_ref().unwrap();
        let allocator = self.allocator.as_ref().unwrap();
        let queue = self.present_queue.as_ref().unwrap();
        let command_buffer = self.setup_command_buffer.as_ref().unwrap();

        let entities = self.entities.as_ref().unwrap();
        let entity = &entities[0];

        let vertices = entity.vertices();
        let buffer_size = std::mem::size_of::<Vertex>() as u64 * vertices.len() as u64;

        let mut staging_buffer = Buffer::new(
            device.clone(),
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            allocator.clone(),
            MemoryLocation::CpuToGpu
        );

        staging_buffer.set_data(&vertices);

        let mut vertex_buffer = Buffer::new(
            device.clone(),
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            allocator.clone(),
            MemoryLocation::GpuOnly
        );

        vertex_buffer.transform_from(&queue, &command_buffer ,&staging_buffer);

        self.vertex_buffers = Some(vec![vertex_buffer]);
    }

    fn create_index_buffer(&mut self) {
        let device = self.device.as_ref().unwrap();
        let allocator = self.allocator.as_ref().unwrap();
        let queue = self.present_queue.as_ref().unwrap();
        let command_buffer = self.setup_command_buffer.as_ref().unwrap();

        let entities = self.entities.as_ref().unwrap();
        let entity = &entities[0];

        let indices = entity.indices();
        let buffer_size = std::mem::size_of::<u16>() as u64 * indices.len() as u64;

        let mut staging_buffer = Buffer::new(
            device.clone(),
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            allocator.clone(),
            MemoryLocation::CpuToGpu
        );

        staging_buffer.set_data(&indices);

        let mut vertex_buffer = Buffer::new(
            device.clone(),
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            allocator.clone(),
            MemoryLocation::GpuOnly
        );

        vertex_buffer.transform_from(&queue, &command_buffer ,&staging_buffer);

        self.index_buffers = Some(vec![vertex_buffer]);
    }

    fn create_uniform_buffers(&mut self) {
        let device = self.device.as_ref().unwrap();
        let allocator = self.allocator.as_ref().unwrap();
        let present_image_size = self.swapchain_image_count.unwrap();

        let camera = self.camera.as_ref().unwrap();
        let entity = &self.entities.as_ref().unwrap()[0];
        let ubo = camera.data(&entity.transform());

        fn calc_vertices(ubo: &UniformBufferObject, vertices: &Vec<Vertex>) {
            for v in vertices {
                let pos = na::Vector4::from(v.pos);
                let pos4d = (ubo.cam4_trans * pos + ubo.cam4_col) / ((ubo.cam4_row.transpose() * pos)[0] + ubo.cam4_const);
                let pos4d = pos4d / pos4d[3];
                let gl_pos = ubo.cam3_trans * pos4d;
                println!("{:?}", gl_pos / gl_pos[3]);
            }
        }

        calc_vertices(&ubo, &self.entities.as_ref().unwrap()[0].vertices());

        let mut uniform_buffers = Vec::with_capacity(present_image_size);
        
        for _ in 0..present_image_size {
            let buffer_size = std::mem::size_of::<UniformBufferObject>() as u64;
        
            let mut uniform_buffer = Buffer::new(
                device.clone(),
                buffer_size,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                allocator.clone(),
                MemoryLocation::CpuToGpu
            );

            uniform_buffer.set_data(&vec![ubo]);

            uniform_buffers.push(uniform_buffer);
        }

        self.uniform_buffers = Some(uniform_buffers);
    }

    fn create_allocator(&mut self) {

        let instance = self.instance.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();
        let physical_device = self.physical_device.as_ref().unwrap();

        let allocator = {
            Allocator::new(&AllocatorCreateDesc {
                instance: instance.clone(),
                device: device.clone(),
                physical_device: *physical_device,
                debug_settings: Default::default(),
                buffer_device_address: false,
            }).expect("Couldn't create allocator")
        };

        let allocator = Arc::new(Mutex::new(allocator));

        self.allocator = Some(allocator);
    }

    // fn create_egui_integration(&mut self) {
    //     let surface_resolution = self.surface_resolution.as_ref().unwrap();
    //     let window = self.window.as_ref().unwrap();
    //     let device = self.device.as_ref().unwrap();
    //     let surface_format = self.surface_format.as_ref().unwrap();
    //     let swapchain_loader = self.swapchain_loader.as_ref().unwrap();
    //     let swapchain = self.swapchain.as_ref().unwrap();
    //     let allocator = self.allocator.as_ref().unwrap();
    //     let egui_integration = Integration::new(
    //         surface_resolution.width,
    //         surface_resolution.height,
    //         window.scale_factor(),
    //         egui::FontDefinitions::default(),
    //         egui::Style::default(),
    //         device.clone(),
    //         Arc::clone(&allocator),
    //         swapchain_loader.clone(),
    //         swapchain.clone(),
    //         surface_format.clone(),
    //     );
    //     self.egui_integration = Some(egui_integration);
    // }
    // fn egui_ui(&mut self, command_buffer: vk::CommandBuffer, image_index: usize) {
    //     let egui_integration = self.egui_integration.as_mut().unwrap();
    //     let window = self.window.as_ref().unwrap();
    //     // match self.theme {
    //     //     EguiTheme::Dark => self
    //     //         .egui_integration
    //     //         .context()
    //     //         .set_visuals(egui::style::Visuals::dark()),
    //     //     EguiTheme::Light => self
    //     //         .egui_integration
    //     //         .context()
    //     //         .set_visuals(egui::style::Visuals::light()),
    //     // }
    //     egui_integration.begin_frame();
    //     egui::SidePanel::left("my_side_panel").show(&egui_integration.context(), |ui| {
    //         ui.heading("Hello");
    //         ui.label("Hello egui!");
    //         ui.separator();
    //         // ui.horizontal(|ui| {
    //         //     ui.label("Theme");
    //         //     let id = ui.make_persistent_id("theme_combo_box_side");
    //         //     egui::ComboBox::from_id_source(id)
    //         //         .selected_text(format!("{:?}", self.theme))
    //         //         .show_ui(ui, |ui| {
    //         //             ui.selectable_value(&mut self.theme, EguiTheme::Dark, "Dark");
    //         //             ui.selectable_value(&mut self.theme, EguiTheme::Light, "Light");
    //         //         });
    //         // });
    //         // ui.separator();
    //         ui.hyperlink("https://github.com/emilk/egui");
    //         ui.separator();
    //         ui.label("Rotation");
    //         // ui.add(egui::widgets::DragValue::new(&mut self.rotation));
    //         // ui.add(egui::widgets::Slider::new(
    //         //     &mut self.rotation,
    //         //     -180.0..=180.0,
    //         // ));
    //         ui.label("Light Position");
    //         // ui.horizontal(|ui| {
    //         //     ui.label("x:");
    //         //     ui.add(egui::widgets::DragValue::new(&mut self.light_position.x));
    //         //     ui.label("y:");
    //         //     ui.add(egui::widgets::DragValue::new(&mut self.light_position.y));
    //         //     ui.label("z:");
    //         //     ui.add(egui::widgets::DragValue::new(&mut self.light_position.z));
    //         // });
    //         ui.separator();
    //         // ui.text_edit_singleline(&mut self.text);
    //     });
    //     // egui::Window::new("My Window")
    //     //     .resizable(true)
    //     //     .scroll(true)
    //     //     .show(&egui_integration.context(), |ui| {
    //     //         ui.heading("Hello");
    //     //         ui.label("Hello egui!");
    //     //         ui.separator();
    //     //         ui.horizontal(|ui| {
    //     //             ui.label("Theme");
    //     //             let id = ui.make_persistent_id("theme_combo_box_window");
    //     //             egui::ComboBox::from_id_source(id)
    //     //                 .selected_text(format!("{:?}", self.theme))
    //     //                 .show_ui(ui, |ui| {
    //     //                     ui.selectable_value(&mut self.theme, EguiTheme::Dark, "Dark");
    //     //                     ui.selectable_value(&mut self.theme, EguiTheme::Light, "Light");
    //     //                 });
    //     //         });
    //     //         ui.separator();
    //     //         ui.hyperlink("https://github.com/emilk/egui");
    //     //         ui.separator();
    //     //         ui.label("Rotation");
    //     //         ui.add(egui::widgets::DragValue::new(&mut self.rotation));
    //     //         ui.add(egui::widgets::Slider::new(
    //     //             &mut self.rotation,
    //     //             -180.0..=180.0,
    //     //         ));
    //     //         ui.label("Light Position");
    //     //         ui.horizontal(|ui| {
    //     //             ui.label("x:");
    //     //             ui.add(egui::widgets::DragValue::new(&mut self.light_position.x));
    //     //             ui.label("y:");
    //     //             ui.add(egui::widgets::DragValue::new(&mut self.light_position.y));
    //     //             ui.label("z:");
    //     //             ui.add(egui::widgets::DragValue::new(&mut self.light_position.z));
    //     //         });
    //     //         ui.separator();
    //     //         ui.text_edit_singleline(&mut self.text);
    //     //     });
    //     let (_, shapes) = egui_integration.end_frame(&window);
    //     let clipped_meshes = egui_integration.context().tessellate(shapes);
    //     egui_integration
    //         .paint(command_buffer, image_index, clipped_meshes);
    // }

    fn update_uniform_buffer(&mut self) {
        let camera = self.camera.as_mut().unwrap();
        let entity = &self.entities.as_ref().unwrap()[0];

        camera.update_view();
        let ubo = camera.data(&entity.transform());

        let uniform_buffers = self.uniform_buffers.as_mut().unwrap();

        for uniform_buffer in uniform_buffers {
            uniform_buffer.set_data(&vec![ubo]);
        }
    }

    fn cleanup_swapchain(&mut self) {
        let device = self.device.as_ref().unwrap();
        let swapchain_loader = self.swapchain_loader.as_ref().unwrap();
        let command_pool = self.command_pool.as_ref().unwrap();
        let allocator = self.allocator.as_ref().unwrap();

        let depth_image_allocation = self.depth_image_allocation.take().unwrap();
        unsafe {
            device.device_wait_idle().unwrap();

            device.destroy_image(self.depth_image.take().unwrap(), None);
            device.destroy_image_view(self.depth_image_view.take().unwrap(), None);
            allocator.lock().unwrap().free(depth_image_allocation).unwrap();

            device.free_command_buffers(*command_pool, self.draw_command_buffers.take().unwrap().as_slice());
            
            for framebuffer in self.framebuffers.take().unwrap() {
                device.destroy_framebuffer(framebuffer, None);
            }

            drop(self.pipeline.take().unwrap());

            for image_view in self.swapchain_image_views.take().unwrap() {
                device.destroy_image_view(image_view, None);
            }

            swapchain_loader.destroy_swapchain(self.swapchain.take().unwrap(), None);

            for uniform_buffer in self.uniform_buffers.take().unwrap() {
                drop(uniform_buffer);
            }

            device.destroy_descriptor_pool(self.descriptor_pool.take().unwrap(), None);
        }
    }

    pub fn update_surface_resolution(&mut self, surface_resolution: vk::Extent2D) {
        self.surface_resolution = Some(surface_resolution);
    }

    pub fn recreate_swapchain(&mut self) {
        let device = self.device.as_ref().unwrap();

        unsafe {
            device.device_wait_idle().unwrap();
        }

        self.cleanup_swapchain();

        self.create_swapchain();
        self.create_image_views();
        self.create_uniform_buffers();

        self.create_descriptor_pool();
        self.create_descriptor_sets();

        self.create_pipeline();
        self.create_depth_resource();

        self.create_framebuffers();
        self.create_draw_command_buffers();

        // let egui_integration = self.egui_integration.as_mut().unwrap();
        // let surface_resolution = self.surface_resolution.as_ref().unwrap();
        // let swapchain = self.swapchain.as_ref().unwrap();
        // let surface_format = self.surface_format.as_ref().unwrap();

        // egui_integration.update_swapchain(
        //     surface_resolution.width,
        //     surface_resolution.height,
        //     swapchain.clone(),
        //     surface_format.clone(),
        // );

    }

    pub fn egui_integration_handle_event<T>(&mut self, winit_event: &winit::event::Event<T>) {
        // let egui_integration = self.egui_integration.as_mut().unwrap();
        // egui_integration.handle_event(winit_event);
    }

    pub fn render(&mut self) {
        self.update_uniform_buffer();

        let device = self.device.as_ref().unwrap();
        let present_queue = self.present_queue.as_ref().unwrap();
        let swapchain_loader = self.swapchain_loader.as_ref().unwrap();
        let swapchain = self.swapchain.as_ref().unwrap();
        let draw_command_buffers = self.draw_command_buffers.as_ref().unwrap();
        let current_frame = self.current_frame.unwrap();
        let max_frames_in_flight = self.max_frames_in_flight.unwrap();

        let render_finished_semaphores = self.render_finished_semaphores.as_ref().unwrap();
        let image_available_semaphores = self.image_available_semaphores.as_ref().unwrap();
        let inflight_fences = self.inflight_fences.as_ref().unwrap();
        let images_inflight = self.images_inflight.as_mut().unwrap();

        unsafe {
            device
                .wait_for_fences(&[inflight_fences[current_frame]], true, std::u64::MAX)
                .expect("Wait for fence failed.");

            let (image_index, _) = swapchain_loader
                .acquire_next_image(
                    *swapchain,
                    std::u64::MAX,
                    image_available_semaphores[current_frame],
                    vk::Fence::null(),
                )
                .expect("Failed to acquire swapchain image.");
            
            if vk::Fence::null() != images_inflight[image_index as usize] {
                let fences = &[images_inflight[image_index as usize]];

                device
                    .wait_for_fences(fences, true, std::u64::MAX)
                    .expect("Failed to wait for fence.");
            }

            images_inflight[image_index as usize] = inflight_fences[current_frame];

            let wait_semaphores = &[image_available_semaphores[current_frame]];
            let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = &[draw_command_buffers[image_index as usize]];
            let signal_semaphores = &[render_finished_semaphores[current_frame]];

            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(wait_semaphores)
                .wait_dst_stage_mask(wait_stages)
                .command_buffers(command_buffers)
                .signal_semaphores(signal_semaphores);
            
            device
                .reset_fences(&[inflight_fences[current_frame]])
                .expect("Failed to reset fences.");

            device
                .queue_submit(
                    *present_queue,
                    &[submit_info.build()],
                    inflight_fences[current_frame],
                )
                .expect("Failed to submit draw command.");
            
            let swapchains = &[*swapchain];
            let image_indices = &[image_index];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(signal_semaphores)
                .swapchains(swapchains)
                .image_indices(image_indices);

            swapchain_loader
                .queue_present(*present_queue, &present_info)
                .expect("Failed to present swapchain image.");
        }

        self.current_frame = Some((current_frame + 1) % max_frames_in_flight);
    }

    pub fn camera(&mut self) -> &mut Camera {
        self.camera.as_mut().unwrap()
    }

}

impl Drop for App {
    fn drop(&mut self) {
        self.cleanup_swapchain();

        let device = self.device.as_ref().unwrap();
        let surface_loader = self.surface_loader.as_ref().unwrap();
        let debug_utils_loader = self.debug_utils_loader.as_ref().unwrap();

        let allocator = self.allocator.take().unwrap();
        unsafe {
            device.device_wait_idle().unwrap();
            // drop(self.egui_integration.take().unwrap());
            drop(allocator);

            drop(self.index_buffers.take().unwrap());
            drop(self.vertex_buffers.take().unwrap());

            for descriptor_set_layout in self.descriptor_set_layouts.take().unwrap() {
                device.destroy_descriptor_set_layout(descriptor_set_layout, None);
            }

            for render_finished_semaphore in self.render_finished_semaphores.take().unwrap() {
                device.destroy_semaphore(render_finished_semaphore, None);
            }

            for image_available_semaphore in self.image_available_semaphores.take().unwrap() {
                device.destroy_semaphore(image_available_semaphore, None);
            }

            for inflight_fence in self.inflight_fences.take().unwrap() {
                device.destroy_fence(inflight_fence, None);
            }

            device.destroy_command_pool(self.command_pool.take().unwrap(), None);

            device.destroy_device(None);

            surface_loader.destroy_surface(self.surface.take().unwrap(), None);

            debug_utils_loader.destroy_debug_utils_messenger(self.debug_utils_messenger.take().unwrap(), None);

            self.instance.take().unwrap().destroy_instance(None);

        }
    }
}