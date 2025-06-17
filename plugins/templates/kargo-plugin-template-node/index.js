/**
 * {{plugin_description}}
 * 
 * @author {{author_name}} <{{author_email}}>
 */

const fs = require('fs');
const path = require('path');

class {{plugin_name | pascal_case}}Plugin {
  getCommand() {
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
          name: "format",
          short: "f",
          long: "format",
          help: "Output format (json, text, csv)",
          required: false,
          takesValue: true
        }
        // TODO: Add more arguments as needed
      ]
    };
  }

  execute(args) {
    try {
      // TODO: Implement your plugin logic here
      const input = args.input || "default";
      const output = args.output;
      const format = args.format || "text";

      let result = `Hello from {{plugin_name}}! Processing: ${input}`;
      
      if (output) {
        // Example: write to file
        if (format === "json") {
          fs.writeFileSync(output, JSON.stringify({ message: result }, null, 2));
          result += `\nJSON output written to: ${output}`;
        } else {
          fs.writeFileSync(output, result);
          result += `\nOutput written to: ${output}`;
        }
      }

      return {
        success: true,
        output: result
      };
    } catch (error) {
      return {
        success: false,
        error: error.message || String(error)
      };
    }
  }

  getMetadata() {
    return {
      name: "{{plugin_name}}",
      version: "0.1.0",
      description: "{{plugin_description}}",
      author: "{{author_name}}",
      language: "node"
    };
  }
}

// Create plugin instance
const plugin = new {{plugin_name | pascal_case}}Plugin();

// WASM exports
global.get_command = function() {
  return JSON.stringify(plugin.getCommand());
};

global.execute = function(argsJson) {
  const args = JSON.parse(argsJson);
  const result = plugin.execute(args);
  return JSON.stringify(result);
};

global.get_metadata = function() {
  return JSON.stringify(plugin.getMetadata());
};

// For testing
if (require.main === module) {
  console.log("Testing {{plugin_name}} plugin...");
  console.log("Command:", get_command());
  console.log("Metadata:", get_metadata());
  
  const testArgs = { input: "test.txt", format: "json" };
  console.log("Execute result:", execute(JSON.stringify(testArgs)));
}

module.exports = { {{plugin_name | pascal_case}}Plugin, get_command, execute, get_metadata };