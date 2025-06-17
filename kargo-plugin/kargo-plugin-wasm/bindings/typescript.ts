/**
 * Kargo WASM Plugin Interface for TypeScript/Node.js
 * 
 * Example usage:
 * ```typescript
 * import { KargoPlugin, CommandDefinition, PluginMetadata } from 'kargo-plugin';
 * 
 * class MyPlugin implements KargoPlugin {
 *   getCommand(): CommandDefinition {
 *     return {
 *       name: "my-plugin",
 *       about: "My awesome plugin",
 *       args: [{
 *         name: "input",
 *         short: "i",
 *         long: "input",
 *         help: "Input file",
 *         required: true,
 *         takesValue: true
 *       }]
 *     };
 *   }
 *   
 *   execute(args: Record<string, any>): ExecutionResult {
 *     // Your plugin logic here
 *     return { success: true, output: "Done!" };
 *   }
 * }
 * ```
 */

export interface ArgDefinition {
  name: string;
  short?: string;
  long?: string;
  help: string;
  required: boolean;
  takesValue: boolean;
}

export interface CommandDefinition {
  name: string;
  about: string;
  args: ArgDefinition[];
}

export interface PluginMetadata {
  name: string;
  version: string;
  description: string;
  author: string;
  language: string;
}

export interface ExecutionResult {
  success: boolean;
  output?: string;
  error?: string;
}

export interface KargoPlugin {
  getCommand(): CommandDefinition;
  execute(args: Record<string, any>): ExecutionResult;
  getMetadata(): PluginMetadata;
}

// WASM export functions
let pluginInstance: KargoPlugin | null = null;

export function setPlugin(plugin: KargoPlugin) {
  pluginInstance = plugin;
}

export function get_command(): string {
  if (!pluginInstance) throw new Error("Plugin not initialized");
  return JSON.stringify(pluginInstance.getCommand());
}

export function execute(argsJson: string): string {
  if (!pluginInstance) throw new Error("Plugin not initialized");
  const args = JSON.parse(argsJson);
  const result = pluginInstance.execute(args);
  return JSON.stringify(result);
}

export function get_metadata(): string {
  if (!pluginInstance) throw new Error("Plugin not initialized");
  return JSON.stringify(pluginInstance.getMetadata());
}