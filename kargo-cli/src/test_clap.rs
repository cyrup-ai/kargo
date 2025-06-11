#[cfg(test)]
mod tests {
    use clap::Command;

    #[test]
    fn test_clap_string_types() {
        // Test 1: Using &str (this should work)
        let _cmd1 = Command::new("test_app");
        
        // Test 2: Using String directly - let's see if this compiles
        let app_name = String::from("test_app");
        let _cmd2 = Command::new(app_name); // This might fail
        
        // Test 3: Using String with as_str()
        let app_name2 = String::from("test_app2");
        let _cmd3 = Command::new(app_name2.as_str());
        
        // Test 4: Using &String
        let app_name3 = String::from("test_app3");
        let _cmd4 = Command::new(&app_name3);
    }
}