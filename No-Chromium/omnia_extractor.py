import urllib.request
import os
import re
import time

class V8OmniaExtractor:
    """
    Recursive extractor to pull the entire V8 'Bible' from official sources.
    It follows includes and crawls directories to ensure total extraction.
    """
    def __init__(self, base_dir="No-Chromium/extracted_data"):
        self.base_dir = base_dir
        self.v8_raw_root = "https://raw.githubusercontent.com/v8/v8/main/"
        self.visited = set()
        
        if not os.path.exists(self.base_dir):
            os.makedirs(self.base_dir)

    def download_file(self, remote_path):
        """Downloads a single file from the V8 repo."""
        if remote_path in self.visited:
            return None
        self.visited.add(remote_path)
        
        url = self.v8_raw_root + remote_path
        local_filename = remote_path.replace("/", "_")
        local_path = os.path.join(self.base_dir, local_filename)
        
        print(f"[*] Crawling: {remote_path}...")
        try:
            req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
            with urllib.request.urlopen(req) as response:
                content = response.read().decode('utf-8', errors='ignore')
                with open(local_path, "w", encoding='utf-8') as f:
                    f.write(content)
                return content
        except Exception as e:
            print(f"    [!] Error: {e}")
            return None

    def crawl_includes(self, file_path, content):
        """Finds all #include "v8-..." in the content and downloads them."""
        # Regex for V8-specific includes
        includes = re.findall(r'#include "(v8-[^"]+\.h)"', content)
        for inc in includes:
            # Most includes are in the same 'include/' directory
            full_path = f"include/{inc}"
            inc_content = self.download_file(full_path)
            if inc_content:
                # Recursively follow further includes
                self.crawl_includes(full_path, inc_content)

    def crawl_torque_builtins(self):
        """
        Since we can't 'list' GitHub directories via raw URL, 
        we target the most critical Torque files known in V8 architecture.
        """
        core_builtins = [
            "src/builtins/array-map.tq",
            "src/builtins/array-foreach.tq",
            "src/builtins/array-reduce.tq",
            "src/builtins/array-filter.tq",
            "src/builtins/array-slice.tq",
            "src/builtins/object.tq",
            "src/builtins/string-substring.tq",
            "src/builtins/boolean.tq",
            "src/builtins/number.tq",
            "src/builtins/promise-abstract-operations.tq",
            "src/builtins/promise-constructor.tq"
        ]
        
        print("[*] Starting extraction of the 'Bible' (Torque Algorithms)...")
        for tq in core_builtins:
            self.download_file(tq)

    def run(self):
        # 1. Start from the Genesis: v8.h
        print("[*] --- PHASE 1: API SURFACE EXTRACTION (Genesis) ---")
        genesis_content = self.download_file("include/v8.h")
        if genesis_content:
            self.crawl_includes("include/v8.h", genesis_content)
        
        # 2. Extract the algorithms
        print("\n[*] --- PHASE 2: ALGORITHMIC ADN EXTRACTION (The Laws) ---")
        self.crawl_torque_builtins()
        
        print(f"\n[+] EXTRACTION COMPLETE. {len(self.visited)} files recovered.")
        print(f"[+] All data integrated in {self.base_dir}")

if __name__ == "__main__":
    extractor = V8OmniaExtractor()
    extractor.run()
