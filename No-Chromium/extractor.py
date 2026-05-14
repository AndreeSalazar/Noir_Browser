import urllib.request
import os
import re

class ChromiumExtractor:
    """
    Automates the extraction of Web Standards and Chromium specifications.
    This data will be used to build the 'No-Chromium' engine base.
    """
    def __init__(self, base_dir="No-Chromium/extracted_data"):
        self.base_dir = base_dir
        self.sources = {
            "DOM": "https://raw.githubusercontent.com/whatwg/dom/main/dom.bs",
            "HTML": "https://raw.githubusercontent.com/whatwg/html/main/source",
            "V8_ArrayMap": "https://raw.githubusercontent.com/v8/v8/main/src/builtins/array-map.tq",
            "V8_ArrayForEach": "https://raw.githubusercontent.com/v8/v8/main/src/builtins/array-foreach.tq",
            "V8_ArraySlice": "https://raw.githubusercontent.com/v8/v8/main/src/builtins/array-slice.tq",
            "V8_Object": "https://raw.githubusercontent.com/v8/v8/main/src/builtins/object.tq",
            "V8_Base_H": "https://raw.githubusercontent.com/v8/v8/main/include/v8.h"
        }
        if not os.path.exists(self.base_dir):
            os.makedirs(self.base_dir)

    def extract_all(self):
        """
        Downloads all raw IDL files from the official sources.
        """
        print("[*] Starting mass extraction of WebIDL 'ADN'...")
        
        for name, url in self.sources.items():
            print(f"[*] Fetching {name} from {url}...")
            try:
                req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
                with urllib.request.urlopen(req) as response:
                    content = response.read().decode('utf-8', errors='ignore')
                    # Logic for IDL files (Bikeshed)
                    if url.endswith(".bs") or "source" in url:
                        idl_blocks = re.findall(r'<pre class="?idl"?>([\s\S]*?)</pre>', content)
                        output_file = os.path.join(self.base_dir, f"{name.lower()}.idl")
                        with open(output_file, "w", encoding='utf-8') as f:
                            for block in idl_blocks:
                                clean = re.sub(r'<[^>]*>', '', block)
                                clean = clean.replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&")
                                f.write(clean.strip() + "\n\n")
                        print(f"[+] Extracted {len(idl_blocks)} IDL interfaces to {output_file}")
                    
                    # Logic for V8 Torque or Header files
                    else:
                        extension = "tq" if url.endswith(".tq") else "h"
                        output_file = os.path.join(self.base_dir, f"{name.lower()}.{extension}")
                        with open(output_file, "w", encoding='utf-8') as f:
                            f.write(content)
                        print(f"[+] Saved V8 ADN to {output_file}")
            except Exception as e:
                print(f"[!] Failed to fetch {name}: {e}")

if __name__ == "__main__":
    extractor = ChromiumExtractor()
    extractor.extract_all()
