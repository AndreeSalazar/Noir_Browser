use ash::{vk, Entry, Instance};
use ash::extensions::ext::DebugUtils;
use raw_window_handle::HasRawDisplayHandle;
use std::ffi::{CStr, CString};
use std::os::raw::c_void;

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message = if callback_data.p_message.is_null() {
        std::borrow::Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };
    if message_severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        // Ignorar advertencia específica de ash
        if !message.contains("vkGetPhysicalDevicePresentRectanglesKHR") {
            println!("[Vulkan Debug] {}", message);
        }
    }
    vk::FALSE
}

pub struct VulkanInstance {
    pub entry: Entry,
    pub instance: Instance,
    pub debug_utils_loader: DebugUtils,
    pub debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl VulkanInstance {
    pub fn new(window: &winit::window::Window) -> Self {
        unsafe {
            println!("[*] Inicializando Ash Entry...");
            let entry = Entry::load().expect("Failed to load Vulkan driver");

            println!("[*] Configuracion de Validation Layers y Extensiones...");
            let app_name = CString::new("No-Chromium").unwrap();
            let engine_name = CString::new("No-Chromium Engine").unwrap();
            let app_info = vk::ApplicationInfo::builder()
                .application_name(&app_name)
                .application_version(vk::make_api_version(0, 1, 0, 0))
                .engine_name(&engine_name)
                .engine_version(vk::make_api_version(0, 1, 0, 0))
                .api_version(vk::make_api_version(0, 1, 3, 0));

            let layer_names = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
            let layers_names_raw: Vec<*const i8> = layer_names.iter().map(|raw_name| raw_name.as_ptr()).collect();

            let mut extension_names = ash_window::enumerate_required_extensions(window.raw_display_handle()).unwrap().to_vec();
            extension_names.push(DebugUtils::name().as_ptr());

            let mut debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
                .message_type(vk::DebugUtilsMessageTypeFlagsEXT::GENERAL | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE)
                .pfn_user_callback(Some(vulkan_debug_callback));

            let create_info = vk::InstanceCreateInfo::builder()
                .application_info(&app_info)
                .enabled_layer_names(&layers_names_raw)
                .enabled_extension_names(&extension_names)
                .push_next(&mut debug_create_info);

            println!("[*] Creando Instancia Vulkan...");
            let instance = entry.create_instance(&create_info, None).expect("Failed to create Vulkan Instance");

            let debug_utils_loader = DebugUtils::new(&entry, &instance);
            let debug_messenger = debug_utils_loader.create_debug_utils_messenger(&debug_create_info, None).unwrap();

            Self {
                entry,
                instance,
                debug_utils_loader,
                debug_messenger,
            }
        }
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe {
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.instance.destroy_instance(None);
        }
    }
}
