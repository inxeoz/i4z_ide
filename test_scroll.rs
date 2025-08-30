// This is a test file with many lines to test scrolling functionality
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

fn main() {
    println!("Starting the scroll test program...");
    
    // Line 10: Create a hashmap with some data
    let mut scores = HashMap::new();
    scores.insert("Alice", 95);
    scores.insert("Bob", 87);
    scores.insert("Charlie", 92);
    scores.insert("Diana", 88);
    
    // Line 17: Print the scores
    for (name, score) in &scores {
        println!("{}: {}", name, score);
    }
    
    // Line 22: Some function definitions
    fn calculate_average(numbers: &[i32]) -> f64 {
        let sum: i32 = numbers.iter().sum();
        sum as f64 / numbers.len() as f64
    }
    
    // Line 28: Test the function
    let test_numbers = vec![10, 20, 30, 40, 50];
    let avg = calculate_average(&test_numbers);
    println!("Average: {:.2}", avg);
    
    // Line 33: File operations example
    let file_path = "test_output.txt";
    match write_to_file(file_path, "Hello, world!") {
        Ok(()) => println!("File written successfully"),
        Err(e) => println!("Error writing file: {}", e),
    }
    
    // Line 40: Read the file back
    match read_from_file(file_path) {
        Ok(content) => println!("File content: {}", content),
        Err(e) => println!("Error reading file: {}", e),
    }
    
    // Line 46: More lines for scrolling test
    let mut counter = 0;
    loop {
        counter += 1;
        if counter > 10 {
            break;
        }
        println!("Counter: {}", counter);
    }
    
    // Line 55: Vector operations
    let mut vec = Vec::new();
    for i in 1..=20 {
        vec.push(i * i); // Square numbers
    }
    
    // Line 61: Print squares
    for (index, value) in vec.iter().enumerate() {
        println!("Square of {}: {}", index + 1, value);
    }
    
    // Line 66: Struct definition
    struct Person {
        name: String,
        age: u32,
        email: String,
    }
    
    // Line 73: Create some people
    let people = vec![
        Person {
            name: "John".to_string(),
            age: 30,
            email: "john@example.com".to_string(),
        },
        Person {
            name: "Jane".to_string(),
            age: 25,
            email: "jane@example.com".to_string(),
        },
        Person {
            name: "Bob".to_string(),
            age: 35,
            email: "bob@example.com".to_string(),
        },
    ];
    
    // Line 90: Print people info
    for person in &people {
        println!("Name: {}, Age: {}, Email: {}", person.name, person.age, person.email);
    }
    
    println!("End of scroll test program");
}

// Line 97: Helper functions
fn write_to_file(path: &str, content: &str) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn read_from_file(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// Line 109: End of file - this should be around line 109 with plenty of content to scroll through