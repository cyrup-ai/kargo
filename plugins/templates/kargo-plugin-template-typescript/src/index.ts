/**
 * {{plugin_description}}
 * 
 * @author {{author_name}} <{{author_email}}>
 */

import { 
  KargoPlugin, 
  CommandDefinition, 
  ArgDefinition, 
  PluginMetadata, 
  ExecutionResult 
} from '../kargo-plugin-wasm/bindings/typescript';

export class {{plugin_name | pascal_case}}Plugin implements KargoPlugin {
  getCommand(): CommandDefinition {
    return {
      name: "{{plugin_name}}",
      about: "{{plugin_description}}",
      args: [
        {
          name: "input",
          short: "i",
          long: "input",
          help: "Input file or value",
          required: false,
          takesValue: true
        },
        {
          name: "output",
          short: "o",
          long: "output",
          help: "Output file",
          required: false,
          takesValue: true
        },
        {
          name: "verbose",
          short: "v",
          long: "verbose",
          help: "Enable verbose output",
          required: false,
          takesValue: false
        }
        // TODO: Add more arguments as needed
      ]
    };
  }

  execute(args: Record<string, any>): ExecutionResult {
    try {
      // TODO: Implement your plugin logic here
      const input = args.input || "default";
      const output = args.output;
      const verbose = args.verbose || false;

      let result = `Hello from {{plugin_name}}! Processing: ${input}`;
      
      if (output) {
        result += `\nOutput will be written to: ${output}`;
      }
      
      if (verbose) {
        result += "\n[Verbose mode enabled]";
        console.log("Debug info:", args);
      }

      return {
        success: true,
        output: result
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error)
      };
    }
  }

  getMetadata(): PluginMetadata {
    return {
      name: "{{plugin_name}}",
      version: "0.1.0",
      description: "{{plugin_description}}",
      author: "{{author_name}}",
      language: "typescript"
    };
  }
}

// Create plugin instance
const plugin = new {{plugin_name | pascal_case}}Plugin();

// WASM exports
export function get_command(): string {
  return JSON.stringify(plugin.getCommand());
}

export function execute(argsJson: string): string {
  const args = JSON.parse(argsJson);
  const result = plugin.execute(args);
  return JSON.stringify(result);
}

export function get_metadata(): string {
  return JSON.stringify(plugin.getMetadata());
}

// For testing
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { {{plugin_name | pascal_case}}Plugin, get_command, execute, get_metadata };
}