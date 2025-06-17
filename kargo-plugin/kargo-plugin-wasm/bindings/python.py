"""
Kargo WASM Plugin Interface for Python

Example usage:
    from kargo_plugin import KargoPlugin, CommandDefinition, ArgDefinition
    
    class MyPlugin(KargoPlugin):
        def get_command(self):
            return CommandDefinition(
                name="my-plugin",
                about="My awesome plugin",
                args=[
                    ArgDefinition(
                        name="input",
                        short="i",
                        long="input",
                        help="Input file",
                        required=True,
                        takes_value=True
                    )
                ]
            )
        
        def execute(self, args):
            # Your plugin logic here
            return {"success": True, "output": "Done!"}
"""

import json
from abc import ABC, abstractmethod
from dataclasses import dataclass, asdict
from typing import List, Optional, Dict, Any

@dataclass
class ArgDefinition:
    name: str
    help: str
    required: bool = False
    takes_value: bool = False
    short: Optional[str] = None
    long: Optional[str] = None

@dataclass
class CommandDefinition:
    name: str
    about: str
    args: List[ArgDefinition]

@dataclass
class PluginMetadata:
    name: str
    version: str
    description: str
    author: str
    language: str = "python"

class KargoPlugin(ABC):
    @abstractmethod
    def get_command(self) -> CommandDefinition:
        pass
    
    @abstractmethod
    def execute(self, args: Dict[str, Any]) -> Dict[str, Any]:
        pass
    
    @abstractmethod
    def get_metadata(self) -> PluginMetadata:
        pass

# WASM export functions
def get_command() -> str:
    plugin = _get_plugin_instance()
    cmd = plugin.get_command()
    return json.dumps(asdict(cmd))

def execute(args_json: str) -> str:
    plugin = _get_plugin_instance()
    args = json.loads(args_json)
    result = plugin.execute(args)
    return json.dumps(result)

def get_metadata() -> str:
    plugin = _get_plugin_instance()
    metadata = plugin.get_metadata()
    return json.dumps(asdict(metadata))