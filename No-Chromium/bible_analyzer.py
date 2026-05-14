import os
import re
import json

class BibleAnalyzer:
    """
    Parses the extracted V8 ADN to create a Reconstruction Map for Rust.
    It identifies core classes, methods, and algorithmic logic.
    """
    def __init__(self, data_dir="No-Chromium/extracted_data"):
        self.data_dir = data_dir
        self.reconstruction_map = {
            "core_classes": [],
            "algorithms": [],
            "memory_management": [],
            "js_builtins": {}
        }

    def analyze_headers(self):
        """Extracts class definitions and public APIs from .h files."""
        print("[*] Analyzing V8 Headers (The Architecture)...")
        for filename in os.listdir(self.data_dir):
            if filename.endswith(".h"):
                path = os.path.join(self.data_dir, filename)
                with open(path, "r", encoding='utf-8', errors='ignore') as f:
                    content = f.read()
                    
                    # Find classes
                    classes = re.findall(r'class V8_EXPORT ([A-Za-z0-9_]+)', content)
                    for cls in classes:
                        self.reconstruction_map["core_classes"].append({
                            "name": cls,
                            "source": filename
                        })

    def analyze_torque(self):
        """Extracts logic and macros from .tq files."""
        print("[*] Analyzing V8 Torque (The Algorithms)...")
        for filename in os.listdir(self.data_dir):
            if filename.endswith(".tq"):
                path = os.path.join(self.data_dir, filename)
                with open(path, "r", encoding='utf-8', errors='ignore') as f:
                    content = f.read()
                    
                    # Find macros/functions in Torque
                    macros = re.findall(r'(?:macro|transitioning builtin) ([A-Za-z0-9_]+)', content)
                    self.reconstruction_map["js_builtins"][filename] = macros
                    
                    # Search for key algorithmic hints
                    if "while" in content or "for" in content:
                        self.reconstruction_map["algorithms"].append({
                            "type": "Loop-based optimization",
                            "source": filename
                        })

    def generate_report(self):
        """Generates the final 'Bible' reconstruction map."""
        output_path = "No-Chromium/v8_reconstruction_map.json"
        with open(output_path, "w", encoding='utf-8') as f:
            json.dump(self.reconstruction_map, f, indent=4)
        
        # Also create a readable Markdown summary
        summary_path = "No-Chromium/THE_BIBLE_SUMMARY.md"
        with open(summary_path, "w", encoding='utf-8') as f:
            f.write("# THE BIBLE OF V8: RECONSTRUCTION MAP\n\n")
            f.write(f"Total Files Analyzed: {len(os.listdir(self.data_dir))}\n\n")
            
            f.write("## 1. Core Classes (Architecture)\n")
            for cls in self.reconstruction_map["core_classes"][:20]: # Show top 20
                f.write(f"- **{cls['name']}** (Source: {cls['source']})\n")
            
            f.write("\n## 2. JS Built-ins (Logic)\n")
            for src, macros in list(self.reconstruction_map["js_builtins"].items())[:10]:
                f.write(f"### {src}\n")
                for m in macros[:5]:
                    f.write(f"- `{m}`\n")
        
        print(f"[+] Bible Analysis Complete. Results in {output_path} and {summary_path}")

if __name__ == "__main__":
    analyzer = BibleAnalyzer()
    analyzer.analyze_headers()
    analyzer.analyze_torque()
    analyzer.generate_report()
