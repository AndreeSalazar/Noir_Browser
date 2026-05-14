import os

class VulkanBootstrapperGen:
    """
    Generates the core Vulkan bootstrapper for Rust.
    This is the bridge that eliminates the white screen.
    """
    def __init__(self, output_dir="No-Chromium/src/generated_rust"):
        self.output_dir = output_dir

    def generate(self):
        print("[*] Generating Vulkan Bootstrapper (The Bridge to Reality)...")
        rust_code = [
            "// AUTO-GENERATED VULKAN BOOTSTRAPPER",
            "use ash::{vk, Entry, Instance, Device};",
            "use ash_window;",
            "use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};",
            "use winit::window::Window;",
            "",
            "pub struct VulkanApp {",
            "    pub instance: Instance,",
            "    pub device: Device,",
            "    pub surface: vk::SurfaceKHR,",
            "    pub surface_loader: ash::extensions::khr::Surface,",
            "}",
            "",
            "impl VulkanApp {",
            "    pub fn new(window: &Window) -> Self {",
            "        println!(\"[Vulkan] Bootstrapping Hardware Instance...\");",
            "        let entry = unsafe { Entry::load().unwrap() };",
            "        ",
            "        // Create Instance with Window Extensions",
            "        let extensions = ash_window::enumerate_required_extensions(window.raw_display_handle()).unwrap();",
            "        let app_info = vk::ApplicationInfo::builder().api_version(vk::make_api_version(0, 1, 3, 0));",
            "        let create_info = vk::InstanceCreateInfo::builder()",
            "            .application_info(&app_info)",
            "            .enabled_extension_names(extensions);",
            "",
            "        let instance = unsafe { entry.create_instance(&create_info, None).unwrap() };",
            "        let surface = unsafe { ash_window::create_surface(&entry, &instance, window.raw_display_handle(), window.raw_window_handle(), None).unwrap() };",
            "        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);",
            "",
            "        // Select Physical Device (RTX 3060 preference)",
            "        let pdevices = unsafe { instance.enumerate_physical_devices().unwrap() };",
            "        let pdevice = pdevices[0]; // Simplified: Take first available GPU",
            "        ",
            "        // Create Logical Device",
            "        let queue_info = [vk::DeviceQueueCreateInfo::builder()",
            "            .queue_family_index(0)",
            "            .queue_priorities(&[1.0])",
            "            .build()];",
            "        let device_create_info = vk::DeviceCreateInfo::builder()",
            "            .queue_create_infos(&queue_info);",
            "        ",
            "        let device = unsafe { instance.create_device(pdevice, &device_create_info, None).unwrap() };",
            "",
            "        println!(\"[Vulkan] Hardware Bridge Established Successfully.\");",
            "        Self { instance, device, surface, surface_loader }",
            "    }",
            "}",
        ]

        with open(os.path.join(self.output_dir, "bootstrapper.rs"), "w", encoding='utf-8') as f:
            f.write("\n".join(rust_code))

        # Update mod.rs
        with open(os.path.join(self.output_dir, "mod.rs"), "a") as f:
            f.write("pub mod bootstrapper;\n")

        print("[+] Vulkan Bootstrapper Ready.")

if __name__ == "__main__":
    gen = VulkanBootstrapperGen()
    gen.generate()
