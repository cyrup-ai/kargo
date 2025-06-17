#!/usr/bin/env python3
"""
{{plugin_description}}

Author: {{author_name}} <{{author_email}}>
"""

import json
from extism_pdk import *

# Plugin state
plugin_metadata = {
    "name": "{{plugin_name}}",
    "version": "0.1.0",
    "description": "{{plugin_description}}",
    "author": "{{author_name}}",
    "language": "python"
}

command_definition = {
    "name": "{{plugin_name}}",
    "about": "{{plugin_description}}",
    "args": [
        {
            "name": "input",
            "short": "i",
            "long": "input",
            "help": "Input file or value",
            "required": False,
            "takes_value": True
        },
        {
            "name": "verbose",
            "short": "v",
            "long": "verbose",
            "help": "Enable verbose output",
            "required": False,
            "takes_value": False
        }
        # TODO: Add more arguments as needed
    ]
}

@extism_fn
def get_command():
    """Return command definition as JSON"""
    output(json.dumps(command_definition))

@extism_fn
def execute():
    """Execute the plugin with given arguments"""
    try:
        # Get input from extism
        args_json = input_str()
        args = json.loads(args_json) if args_json else {}
        
        # TODO: Implement your plugin logic here
        input_value = args.get("input", "default")
        verbose = args.get("verbose", False)
        
        result_text = f"Hello from {{plugin_name}}! Processing: {input_value}"
        if verbose:
            result_text += "\n[Verbose mode enabled]"
        
        result = {
            "success": True,
            "output": result_text
        }
        
        output(json.dumps(result))
    except Exception as e:
        error_result = {
            "success": False,
            "error": str(e)
        }
        output(json.dumps(error_result))

@extism_fn
def get_metadata():
    """Return plugin metadata as JSON"""
    output(json.dumps(plugin_metadata))