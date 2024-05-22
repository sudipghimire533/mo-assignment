use std::collections::{HashMap, VecDeque};

#[derive(Debug)]
pub struct Task {
    pub name: String,
    pub dependencies: Vec<String>,
    pub duration: u32,
}

#[derive(Debug)]
pub struct TaskScheduler {
    pub tasks: HashMap<String, Task>,
    pub first_level_dep: HashMap<String, usize>,
    pub dependencies: HashMap<String, Vec<String>>,
}

impl TaskScheduler {
    pub fn new() -> Self {
        TaskScheduler {
            tasks: HashMap::new(),
            first_level_dep: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, name: &str, dependencies: Vec<&str>, duration: u32) {
        if self.tasks.contains_key(name) {
            panic!("Task with the same name already exists");
        }

        let task = Task {
            name: name.to_string(),
            dependencies: dependencies.iter().map(|&d| d.to_string()).collect(),
            duration,
        };

        self.first_level_dep.insert(name.to_string(), dependencies.len());
        for dep in &task.dependencies {
            self.dependencies
                .entry(dep.to_string())
                .or_default()
                .push(name.to_string());
        }

        self.tasks.insert(name.to_string(), task);
    }

    pub fn schedule_tasks(&self) -> Vec<(String, u32, u32)> {
        let mut zero_in_degree: VecDeque<String> = VecDeque::new();
        let mut in_degree = self.first_level_dep.clone();
        let mut order: Vec<(String, u32, u32)> = Vec::new();
        let mut time: u32 = 0;

        for (task, &degree) in &in_degree {
            if degree == 0 {
                zero_in_degree.push_back(task.clone());
            }
        }

        while let Some(task_name) = zero_in_degree.pop_front() {
            if let Some(task) = self.tasks.get(&task_name) {
                order.push((task_name.clone(), time, task.duration));
                time += task.duration;

                if let Some(neighbors) = self.dependencies.get(&task_name) {
                    for neighbor in neighbors {
                        if let Some(degree) = in_degree.get_mut(neighbor) {
                            *degree -= 1;
                            if *degree == 0 {
                                zero_in_degree.push_back(neighbor.clone());
                            }
                        }
                    }
                }
            }
        }

        if order.len() != self.tasks.len() {
            panic!("Circular dependency detected");
        }

        order
    }
}

#[test]
fn test_non_cylce() {
    let mut scheduler = TaskScheduler::new();
    scheduler.add_task("A", vec![], 3);
    scheduler.add_task("B", vec!["A"], 2);
    scheduler.add_task("C", vec!["A"], 1);
    scheduler.add_task("D", vec!["B", "C"], 4);

    let schedule = scheduler.schedule_tasks();
    assert_eq!(
        schedule,
        vec![
            ("A".to_string(), 0, 3),
            ("B".to_string(), 3, 2),
            ("C".to_string(), 5, 1),
            ("D".to_string(), 6, 4),
        ]
    );
}

#[test]
#[should_panic(expected = "Circular dependency detected")]
fn test_cycle_detection() {
    let mut scheduler = TaskScheduler::new();
    scheduler.add_task("A", vec!["B"], 1);
    scheduler.add_task("B", vec!["C"], 1);
    scheduler.add_task("C", vec!["A"], 1);

    scheduler.schedule_tasks();
}
