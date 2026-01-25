#!/usr/bin/env python3
"""
Fix relative markdown links in docs directory to use GitHub blob URLs
"""
import os
import re
from pathlib import Path

REPO_URL = "https://github.com/ferdiunal/soketi.rs/blob/main"

def fix_links_in_file(file_path):
    """Fix relative markdown links in a single file"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    
    # Get the directory of the current file relative to docs/
    file_dir = os.path.dirname(file_path)
    docs_root = str(Path(file_path).parts[0])  # 'docs'
    
    # Pattern to match markdown links: [text](path.md) or [text](./path.md) or [text](../path.md)
    # But not external links (http/https)
    pattern = r'\[([^\]]+)\]\((?!http)([^\)]+\.md)\)'
    
    def replace_link(match):
        text = match.group(1)
        link = match.group(2)
        
        # Skip if already a full URL
        if link.startswith('http'):
            return match.group(0)
        
        # Resolve the relative path
        if link.startswith('./'):
            link = link[2:]
        
        # Calculate absolute path from docs root
        if link.startswith('../'):
            # Go up directories
            current_dir = file_dir
            while link.startswith('../'):
                link = link[3:]
                current_dir = os.path.dirname(current_dir)
            full_path = os.path.join(current_dir, link)
        else:
            full_path = os.path.join(file_dir, link)
        
        # Normalize path
        full_path = os.path.normpath(full_path)
        
        # Create GitHub blob URL
        github_url = f"{REPO_URL}/{full_path}"
        
        return f"[{text}]({github_url})"
    
    content = re.sub(pattern, replace_link, content)
    
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        return True
    return False

def main():
    """Process all markdown files in docs directory"""
    docs_dir = Path('docs')
    updated_files = []
    
    for md_file in docs_dir.rglob('*.md'):
        if fix_links_in_file(str(md_file)):
            updated_files.append(str(md_file))
            print(f"✓ Updated: {md_file}")
    
    print(f"\n✅ Updated {len(updated_files)} files")
    if updated_files:
        print("\nUpdated files:")
        for f in updated_files:
            print(f"  - {f}")

if __name__ == '__main__':
    main()
