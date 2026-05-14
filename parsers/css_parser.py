import re
import json

class CSSParser:
    """
    Parses CSS strings into a structured representation.
    """
    def __init__(self):
        # Captures: selector { declarations }
        self.rule_re = re.compile(r'([^{]+)\{([^}]+)\}')

    def parse(self, css: str):
        """
        Parses CSS into a list of style rules.
        """
        # Remove comments
        css = re.sub(r'/\*.*?\*/', '', css, flags=re.DOTALL)
        
        rules = []
        for match in self.rule_re.finditer(css):
            selector, decls_raw = match.groups()
            selector = selector.strip()
            
            declarations = {}
            # Split declarations by semicolon
            for decl in decls_raw.split(';'):
                if ':' in decl:
                    prop, val = decl.split(':', 1)
                    declarations[prop.strip().lower()] = val.strip()
            
            if declarations:
                rules.append({
                    "selector": selector,
                    "declarations": declarations
                })
                
        return rules

if __name__ == "__main__":
    parser = CSSParser()
    test_css = """
    body { background-color: #f0f0f0; margin: 0; }
    h1 { color: #2c3e50; font-family: sans-serif; }
    .desc { color: #7f8c8d; line-height: 1.5; }
    """
    css_ir = parser.parse(test_css)
    print(json.dumps(css_ir, indent=4))
