#!/usr/bin/env python3
"""Build script to compile Python plugin to WASM using Extism PDK"""

import subprocess
import sys
import os

plugin_name = "{{plugin_name}}"

def build():
    """Compile Python code to WASM using Extism PDK"""
    print(f"Building {plugin_name} plugin to WASM...")
    
    cmd = [
        "extism-pdk-python",
        "build",
        f"{plugin_name}.py",
        "-o", f"{plugin_name}.wasm"
    ]
    
    try:
        subprocess.run(cmd, check=True)
        print(f"Successfully built {plugin_name}.wasm")
    except subprocess.CalledProcessError as e:
        print(f"Build failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    build()