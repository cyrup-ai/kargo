// {{plugin_description}}
// Author: {{author_name}} <{{author_email}}>

package main

import (
	"encoding/json"
	"fmt"
	"unsafe"
)

// CommandDefinition represents the CLI command structure
type CommandDefinition struct {
	Name  string          `json:"name"`
	About string          `json:"about"`
	Args  []ArgDefinition `json:"args"`
}

// ArgDefinition represents a command argument
type ArgDefinition struct {
	Name       string  `json:"name"`
	Short      *string `json:"short,omitempty"`
	Long       *string `json:"long,omitempty"`
	Help       string  `json:"help"`
	Required   bool    `json:"required"`
	TakesValue bool    `json:"takesValue"`
}

// PluginMetadata contains plugin information
type PluginMetadata struct {
	Name        string `json:"name"`
	Version     string `json:"version"`
	Description string `json:"description"`
	Author      string `json:"author"`
	Language    string `json:"language"`
}

// ExecutionResult represents the plugin execution result
type ExecutionResult struct {
	Success bool    `json:"success"`
	Output  *string `json:"output,omitempty"`
	Error   *string `json:"error,omitempty"`
}

// GetCommand returns the command definition
//export get_command
func GetCommand() *byte {
	shortI := "i"
	longI := "input"
	shortV := "v"
	longV := "verbose"
	
	cmd := CommandDefinition{
		Name:  "{{plugin_name}}",
		About: "{{plugin_description}}",
		Args: []ArgDefinition{
			{
				Name:       "input",
				Short:      &shortI,
				Long:       &longI,
				Help:       "Input file or value",
				Required:   false,
				TakesValue: true,
			},
			{
				Name:       "verbose",
				Short:      &shortV,
				Long:       &longV,
				Help:       "Enable verbose output",
				Required:   false,
				TakesValue: false,
			},
			// TODO: Add more arguments as needed
		},
	}
	
	data, _ := json.Marshal(cmd)
	return &data[0]
}

// Execute runs the plugin with given arguments
//export execute
func Execute(argsPtr *byte, argsLen int) *byte {
	// Convert args from WASM memory
	args := make([]byte, argsLen)
	for i := 0; i < argsLen; i++ {
		args[i] = *(*byte)(unsafe.Pointer(uintptr(unsafe.Pointer(argsPtr)) + uintptr(i)))
	}
	
	var argsMap map[string]interface{}
	if err := json.Unmarshal(args, &argsMap); err != nil {
		errStr := err.Error()
		result := ExecutionResult{
			Success: false,
			Error:   &errStr,
		}
		data, _ := json.Marshal(result)
		return &data[0]
	}
	
	// TODO: Implement your plugin logic here
	input, _ := argsMap["input"].(string)
	if input == "" {
		input = "default"
	}
	
	output := fmt.Sprintf("Hello from {{plugin_name}}! Processing: %s", input)
	
	if verbose, ok := argsMap["verbose"].(bool); ok && verbose {
		output += "\n[Verbose mode enabled]"
	}
	
	result := ExecutionResult{
		Success: true,
		Output:  &output,
	}
	
	data, _ := json.Marshal(result)
	return &data[0]
}

// GetMetadata returns plugin metadata
//export get_metadata
func GetMetadata() *byte {
	metadata := PluginMetadata{
		Name:        "{{plugin_name}}",
		Version:     "0.1.0",
		Description: "{{plugin_description}}",
		Author:      "{{author_name}}",
		Language:    "go",
	}
	
	data, _ := json.Marshal(metadata)
	return &data[0]
}

// Required for TinyGo WASM
func main() {}