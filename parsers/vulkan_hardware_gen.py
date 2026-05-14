import os

class VulkanHardwareGen:
    """
    The High-Level Hardware Linker.
    Generates the Rust code to bridge ash and the OS window.
    """
    def __init__(self, output_dir="No-Chromium/src/generated_rust"):
        self.output_dir = output_dir

    def generate(self):
        print("[*] Generating Vulkan Hardware Linker (The 'Light Switch')...")
        rust_code = [
            "// AUTO-GENERATED VULKAN HARDWARE LINKER",
            "use ash::{vk, Device, Instance};",
            "use ash::extensions::khr::Swapchain;",
            "",
            "pub struct HardwareLinker {",
            "    pub swapchain_loader: Swapchain,",
            "    pub swapchain: vk::SwapchainKHR,",
            "    pub images: Vec<vk::Image>,",
            "}",
            "",
            "impl HardwareLinker {",
            "    pub fn clear_screen(device: &Device, cmd: vk::CommandBuffer, image: vk::Image) {",
            "        let clear_color = vk::ClearColorValue {",
            "            float32: [0.05, 0.05, 0.15, 1.0], // The No-Chromium Premium Dark Blue",
            "        };",
            "        let range = vk::ImageSubresourceRange::builder()",
            "            .aspect_mask(vk::ImageAspectFlags::COLOR)",
            "            .level_count(1)",
            "            .layer_count(1);",
            "",
            "        unsafe {",
            "            device.cmd_clear_color_image(cmd, image, vk::ImageLayout::GENERAL, &clear_color, &[range.build()]);",
            "        }",
            "    }",
            "}",
        ]

        with open(os.path.join(self.output_dir, "vulkan_hardware.rs"), "w", encoding='utf-8') as f:
            f.write("\n".join(rust_code))

        # Update mod.rs
        with open(os.path.join(self.output_dir, "mod.rs"), "a") as f:
            f.write("pub mod vulkan_hardware;\n")

        print("[+] Hardware Linker Code Exported. Ready to light up the GPU.")

if __name__ == "__main__":
    gen = VulkanHardwareGen()
    gen.generate()
