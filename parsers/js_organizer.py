import re

class JSOrganizer:
    """
    Handles extraction and organization of JavaScript before passing it to Rust/QuickJS.
    """
    def __init__(self):
        # Captures content between <script> tags
        self.script_re = re.compile(r'<script[^>]*>(.*?)</script>', re.DOTALL)
        # Captures src attribute in <script src="...">
        self.src_re = re.compile(r'src\s*=\s*["\']([^"\']*)["\']')

    def organize(self, html: str):
        """
        Extracts inline scripts and identifies external scripts.
        """
        scripts = {
            "inline": [],
            "external": []
        }
        
        # Find all script tags
        tag_re = re.compile(r'<script([^>]*)>(.*?)</script>', re.DOTALL)
        
        for match in tag_re.finditer(html):
            attrs_raw, content = match.groups()
            
            src_match = self.src_re.search(attrs_raw)
            if src_match:
                scripts["external"].append(src_match.group(1))
            
            content = content.strip()
            if content:
                scripts["inline"].append(content)
                
        return scripts

if __name__ == "__main__":
    organizer = JSOrganizer()
    test_html = """
    <html>
        <body>
            <script>console.log("Hello from inline JS");</script>
            <script src="external.js"></script>
        </body>
    </html>
    """
    scripts = organizer.organize(test_html)
    import json
    print(json.dumps(scripts, indent=4))
