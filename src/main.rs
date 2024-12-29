use std::collections::HashMap;
use std::fs;
use std::io::{self, Error, ErrorKind};
use serde::{Deserialize, Serialize};
use std::io::Write;

trait TaskManager {
    fn add_task(&mut self, description: String) -> Result<usize, io::Error>;
    fn complete_task(&mut self, id: usize) -> Result<(), io::Error>;
    fn list_tasks(&self) -> Vec<&Task>;
    fn delete_task(&mut self, id: usize) -> Result<(), io::Error>;
}

trait Storage {
    fn save(&self, tasks: &HashMap<usize, Task>) -> Result<(), io::Error>;
    fn load(&self) -> Result<HashMap<usize, Task>, io::Error>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TaskId(usize);

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TaskDescription(String);

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Task {
    id: TaskId,
    description: TaskDescription,
    completed: bool,
}

impl TaskDescription {
    fn new(description: String) -> Result<Self, io::Error> {
        if description.trim().is_empty() {
            return Err(Error::new(ErrorKind::InvalidInput, "Description cannot be empty"));
        }
        Ok(TaskDescription(description))
    }

    fn get(&self) -> &str {
        &self.0
    }
}

struct FileStorage {
    filename: String,
}

impl FileStorage {
    fn new(filename: String) -> Self {
        FileStorage { filename }
    }
}

impl Storage for FileStorage {
    fn save(&self, tasks: &HashMap<usize, Task>) -> Result<(), io::Error> {
        let json = serde_json::to_string(tasks)?;
        fs::write(&self.filename, json)
    }

    fn load(&self) -> Result<HashMap<usize, Task>, io::Error> {
        match fs::read_to_string(&self.filename) {
            Ok(contents) => Ok(serde_json::from_str(&contents)?),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(HashMap::new()),
            Err(e) => Err(e),
        }
    }
}

struct TodoList {
    tasks: HashMap<usize, Task>,
    storage: Box<dyn Storage>,
    next_id: usize,
}

impl TodoList {
    fn new(storage: Box<dyn Storage>) -> Result<Self, io::Error> {
        let tasks = storage.load()?;
        let next_id = tasks.keys().max().map_or(1, |&id| id + 1);
        Ok(TodoList {
            tasks,
            storage,
            next_id,
        })
    }

    fn save(&self) -> Result<(), io::Error> {
        self.storage.save(&self.tasks)
    }
}

impl TaskManager for TodoList {
    fn add_task(&mut self, description: String) -> Result<usize, io::Error> {
        let description = TaskDescription::new(description)?;
        let id = TaskId(self.next_id);
        
        let task = Task {
            id: id.clone(),
            description,
            completed: false,
        };
        
        self.tasks.insert(self.next_id, task);
        self.next_id += 1;
        self.save()?;
        
        Ok(id.0)
    }

    fn complete_task(&mut self, id: usize) -> Result<(), io::Error> {
        match self.tasks.get_mut(&id) {
            Some(task) => {
                task.completed = true;
                self.save()?;
                Ok(())
            }
            None => Err(Error::new(ErrorKind::NotFound, "Task not found")),
        }
    }

    fn list_tasks(&self) -> Vec<&Task> {
        let mut tasks: Vec<&Task> = self.tasks.values().collect();
        tasks.sort_by_key(|task| task.id.0);
        tasks
    }

    fn delete_task(&mut self, id: usize) -> Result<(), io::Error> {
        if self.tasks.remove(&id).is_some() {
            self.save()?;
            Ok(())
        } else {
            Err(Error::new(ErrorKind::NotFound, "Task not found"))
        }
    }
}

fn print_menu() {
    println!("\nTodo List Menu:");
    println!("1. Add task");
    println!("2. List tasks");
    println!("3. Complete task");
    println!("4. Delete task");
    println!("5. Exit");
    print!("\nChoose an option (1-5): ");
    io::stdout().flush().unwrap();
}

fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

fn main() -> Result<(), io::Error> {
    let storage = Box::new(FileStorage::new("todo.json".to_string()));
    let mut todo_list = TodoList::new(storage)?;

    loop {
        print_menu();
        
        let choice = get_input("");
        
        match choice.as_str() {
            "1" => {
                let description = get_input("Enter task description: ");
                match todo_list.add_task(description) {
                    Ok(id) => println!("Added task with ID: {}", id),
                    Err(e) => println!("Error: {}", e),
                }
            },
            "2" => {
                let tasks = todo_list.list_tasks();
                if tasks.is_empty() {
                    println!("No tasks found.");
                } else {
                    println!("\nAll tasks:");
                    for task in tasks {
                        println!(
                            "{}. [{}] {}",
                            task.id.0,
                            if task.completed { "✓" } else { " " },
                            task.description.get()
                        );
                    }
                }
            },
            "3" => {
                let id_str = get_input("Enter task ID to mark as complete: ");
                match id_str.parse::<usize>() {
                    Ok(id) => {
                        match todo_list.complete_task(id) {
                            Ok(_) => println!("Marked task {} as complete", id),
                            Err(e) => println!("Error: {}", e),
                        }
                    },
                    Err(_) => println!("Invalid ID format"),
                }
            },
            "4" => {
                let id_str = get_input("Enter task ID to delete: ");
                match id_str.parse::<usize>() {
                    Ok(id) => {
                        match todo_list.delete_task(id) {
                            Ok(_) => println!("Deleted task {}", id),
                            Err(e) => println!("Error: {}", e),
                        }
                    },
                    Err(_) => println!("Invalid ID format"),
                }
            },
            "5" => {
                println!("Goodbye!");
                break;
            },
            _ => println!("Invalid option, please try again."),
        }
    }

    Ok(())
}