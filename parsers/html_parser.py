import re
import json

class HTMLParser:
    """
    Custom HTML Parser for X-Browser.
    Converts raw HTML into a structured Intermediate Representation (IR).
    """
    def __init__(self):
        # Regex to capture: <closing? tag_name attributes> OR text_content
        self.tag_re = re.compile(r'<(/?)([a-zA-Z0-9]+)([^>]*)>|([^<]+)')

    def parse(self, html: str):
        """
        Parses HTML string into a list of node dictionaries.
        """
        root = {"tag": "root", "children": [], "attributes": {}}
        stack = [root]
        
        for match in self.tag_re.finditer(html):
            is_closing, tag_name, attrs_raw, text = match.groups()
            
            if text:
                text_content = text.strip()
                if text_content:
                    stack[-1]["children"].append({
                        "type": "text", 
                        "content": text_content
                    })
            elif tag_name:
                tag_name = tag_name.lower()
                if is_closing:
                    # Close tag logic
                    if len(stack) > 1 and stack[-1].get("tag") == tag_name:
                        stack.pop()
                else:
                    # Open tag logic
                    attributes = self.parse_attributes(attrs_raw)
                    node = {
                        "type": "element",
                        "tag": tag_name,
                        "attributes": attributes,
                        "children": []
                    }
                    stack[-1]["children"].append(node)
                    
                    # Check for self-closing tags (void elements)
                    void_elements = {"img", "br", "hr", "input", "meta", "link", "area", "base", "col", "embed", "keygen", "param", "source", "track", "wbr"}
                    if tag_name not in void_elements:
                        stack.append(node)
                        
        return root["children"]

    def parse_attributes(self, attrs_raw: str) -> dict:
        """
        Extracts attributes from a tag string.
        Example: class="main" id="header" -> {"class": "main", "id": "header"}
        """
        attrs = {}
        if not attrs_raw:
            return attrs
        
        # Simple attribute regex
        attr_matches = re.findall(r'([a-zA-Z0-9-]+)\s*=\s*["\']?([^"\']*)["\']?', attrs_raw)
        for key, val in attr_matches:
            attrs[key] = val
        return attrs

if __name__ == "__main__":
    parser = HTMLParser()
    test_html = """
    <html>
        <body>
            <h1 id="title">X-Browser Engine</h1>
            <p class="desc">Build from scratch with Python and Rust.</p>
            <img src="logo.png" />
        </body>
    </html>
    """
    dom_ir = parser.parse(test_html)
    print(json.dumps(dom_ir, indent=4))
