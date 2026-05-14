import os

class UniversalElementGen:
    """
    The factory in 'parsers/' that defines all HTML/CSS standards.
    It exports them as native Rust code for the No-Chromium engine.
    """
    def __init__(self, output_dir="No-Chromium/src/generated_rust"):
        self.output_dir = output_dir
        self.html_elements = [
            "div", "span", "p", "a", "img", "video", "canvas", "section", 
            "header", "footer", "nav", "article", "aside", "main", "button",
            "input", "form", "label", "select", "option", "iframe"
        ]
        self.css_properties = [
            "display", "position", "width", "height", "margin", "padding",
            "background-color", "color", "font-size", "border", "flex-direction",
            "justify-content", "align-items", "opacity", "transform", "z-index"
        ]

    def generate_html_elements(self):
        print(f"[*] Generating {len(self.html_elements)} HTML elements for Rust...")
        rust_code = [
            "// AUTO-GENERATED UNIVERSAL HTML ELEMENTS",
            "// This file defines the standard elements used by the parser.",
            "",
            "#[derive(Debug, Clone)]",
            "pub enum HtmlTag {",
        ]
        for tag in self.html_elements:
            rust_code.append(f"    {tag.capitalize()},")
        rust_code.append("    Unknown(String),")
        rust_code.append("}")
        
        rust_code.append("\npub struct HTMLElement {")
        rust_code.append("    pub tag: HtmlTag,")
        rust_code.append("    pub attributes: std::collections::HashMap<String, String>,")
        rust_code.append("    pub id: u64,")
        rust_code.append("}")

        with open(os.path.join(self.output_dir, "html_elements.rs"), "w", encoding='utf-8') as f:
            f.write("\n".join(rust_code))

    def generate_css_engine(self):
        print(f"[*] Generating {len(self.css_properties)} CSS properties for Rust...")
        rust_code = [
            "// AUTO-GENERATED CSS STYLE ENGINE",
            "",
            "#[derive(Debug, Clone)]",
            "pub struct ComputedStyle {",
        ]
        for prop in self.css_properties:
            # Normalize name for Rust struct field
            field_name = prop.replace("-", "_")
            rust_code.append(f"    pub {field_name}: Option<String>,")
        rust_code.append("}")

        with open(os.path.join(self.output_dir, "css_engine.rs"), "w", encoding='utf-8') as f:
            f.write("\n".join(rust_code))

    def run(self):
        self.generate_html_elements()
        self.generate_css_engine()
        
        # Update mod.rs
        with open(os.path.join(self.output_dir, "mod.rs"), "a") as f:
            f.write("pub mod html_elements;\n")
            f.write("pub mod css_engine;\n")
        
        print("[+] Universal Elements Exported to Rust.")

if __name__ == "__main__":
    gen = UniversalElementGen()
    gen.run()
