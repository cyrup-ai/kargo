use clap::Command;
use serde_json::json;

#[test]
fn test_command_from_json() {
    // Test 1: Simple command structure
    let json_str = r#"{"name": "test-command", "about": "A test command"}"#;
    let result: Result<Command, _> = serde_json::from_str(json_str);

    match result {
        Ok(cmd) => {
            println!("Successfully deserialized: {:?}", cmd.get_name());
        }
        Err(e) => {
            println!("Failed to deserialize: {}", e);
        }
    }

    // Test 2: Try with a more complex structure
    let json_value = json!({
        "name": "test-command",
        "about": "A test command",
        "version": "1.0.0"
    });

    let json_str2 =
        serde_json::to_string(&json_value).expect("Failed to serialize JSON value to string");
    let result2: Result<Command, _> = serde_json::from_str(&json_str2);

    match result2 {
        Ok(cmd) => {
            println!("Successfully deserialized complex: {:?}", cmd.get_name());
        }
        Err(e) => {
            println!("Failed to deserialize complex: {}", e);
        }
    }
}

#[test]
fn test_command_new_with_string() {
    // Test different ways to create Command with String
    let name1 = "test1";
    let cmd1 = Command::new(name1);
    assert_eq!(cmd1.get_name(), "test1");

    let name2 = String::from("test2");
    let cmd2 = Command::new(name2.as_str());
    assert_eq!(cmd2.get_name(), "test2");

    let name3 = String::from("test3");
    let cmd3 = Command::new(&name3);
    assert_eq!(cmd3.get_name(), "test3");

    // This should fail to compile if Command::new doesn't accept String
    // let name4 = String::from("test4");
    // let cmd4 = Command::new(name4);
}
