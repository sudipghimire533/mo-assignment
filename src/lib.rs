use std::collections::{HashMap, VecDeque};

/// A task unit.
/// A dependencies must be provided by the user.
#[derive(Debug)]
pub struct Task {
    pub name: String,
    pub dependencies: Vec<String>,
    pub duration: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ScheduleError {
    NoTaskFound,
    CycleDetected,
}

/// Object that act as scheduler
#[derive(Debug)]
pub struct TaskScheduler {
    /// All the task as given by user
    pub tasks: HashMap<String, Task>,
    /// When new task is inserted,
    /// it's count of dependencies is added to this list.
    /// akin to. how many task needed to be comppleted before this can run
    pub first_level_dep: HashMap<String, usize>,
    /// Given a task,
    /// see what other task depends on it.
    /// akin to reverse-dependency
    pub dependents: HashMap<String, Vec<String>>,
}

impl TaskScheduler {
    /// A new empty scheduler
    pub fn new() -> Self {
        TaskScheduler {
            tasks: HashMap::new(),
            first_level_dep: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    /// Add task to the scheduler
    pub fn add_task(&mut self, name: &str, dependencies: Vec<&str>, duration: u32) {
        // Check if task with same name exists already
        if self.tasks.contains_key(name) {
            panic!("Task with the same name already exists");
        }

        // create a task object
        let task = Task {
            name: name.to_string(),
            dependencies: dependencies.iter().map(|&d| d.to_string()).collect(),
            duration,
        };

        // store it's dependencies count
        self.first_level_dep
            .insert(name.to_string(), dependencies.len());

        // a lookup table to get this task's dependents
        // all of it's dependencies will have this task as their dependents
        for dep in &task.dependencies {
            self.dependents
                .entry(dep.to_string())
                .or_default()
                .push(name.to_string());
        }

        self.tasks.insert(name.to_string(), task);
    }

    pub fn schedule_tasks(&self) -> Result<Vec<(String, u32, u32)>, ScheduleError> {
        let mut zero_in_degree: VecDeque<String> = VecDeque::new();
        let mut in_degree = self.first_level_dep.clone();
        let mut order: Vec<(String, u32, u32)> = Vec::new();
        let mut time: u32 = 0;

        // Collect all the task, that have 0 degree
        // i,e it does not have to wait for any other task to run
        // This can be a most bottom task in dependency graph or a task with with dependency
        for (task, &degree) in &in_degree {
            if degree == 0 {
                zero_in_degree.push_back(task.clone());
            }
        }

        // Loop though every task that does not have any dependency.
        // i.e loop from bottom of dependency graph
        while let Some(task_name) = zero_in_degree.pop_front() {
            if let Some(task) = self.tasks.get(&task_name) {
                // We can push directly to the final order for no-dependency tasks
                // this is the section where we add what need to be done exactly
                order.push((task_name.clone(), time, task.duration));
                // TODO: do_something();

                // any task following have to wait for this task to finish.
                // So add that timeline
                //
                // TODO:
                // this assumes the single threading-like behavoiur of executing machine.
                // i.e we have to wait for executing machine to execute current task
                // even if next task is not dependent on current task
                // In practical system,
                // this can be changed to multi-threaded like behaviour
                // i.e if current task is not dependency of next task, run next task
                // in sepearte context ( thread )
                time += task.duration;

                // Get all the tasks which where dependent on this task
                // since this task is complete,
                // we can now execute other task that were directly depending on this task
                if let Some(neighbors) = self.dependents.get(&task_name) {
                    for neighbor in neighbors {
                        if let Some(degree) = in_degree.get_mut(neighbor) {
                            // since we completed the task which was a dependency of neighbour
                            // we can reduce's reighbour's dependency degree by 1
                            *degree -= 1;
                            // check if new depenency degree is 0
                            // if so, it means that neighbour task is no longer dependent on any
                            // other ( i.e it's all dependencies are executed already )
                            // so we can run it. Add this to zero_in_degree variable to preserve
                            // order in next iteration
                            if *degree == 0 {
                                zero_in_degree.push_back(neighbor.clone());
                            }
                        }
                    }
                }
            }
        }

        // all scheduled task are added in order variable
        // and all initial task are stull preserved as-is in self.tasks variable
        // compare the size of those two
        match order.len().cmp(&self.tasks.len()) {
            // Number of task scheduled is less than the initial task count
            // This means some task were not scheduled
            // According to above implementation,
            // only reason this might happen is because a task's dependency degree
            // was never 0
            // This can only happen when the dependency is cyclic then the dependency degree will:
            // =1: dependent to itself
            // >1: dependent to a task which in turn along the way depends on this task
            std::cmp::Ordering::Less => Err(ScheduleError::CycleDetected),

            // This means that all task were scheduled,
            // this is ok result in our case
            std::cmp::Ordering::Equal => Ok(order),

            // This means some tasks were scheduled more than once
            // This will never occur in our case ( single-threaded like environment )
            // so we skip this
            std::cmp::Ordering::Greater => {
                unreachable!()
            }
        }
    }
}

#[test]
fn test_non_cylce() {
    let mut scheduler = TaskScheduler::new();
    scheduler.add_task("D", vec!["B", "C"], 4);
    scheduler.add_task("A", vec![], 3);
    scheduler.add_task("B", vec!["A"], 2);
    scheduler.add_task("C", vec!["A"], 1);

    let schedule = scheduler.schedule_tasks();
    assert_eq!(
        schedule,
        Ok(vec![
            ("A".to_string(), 0, 3),
            ("B".to_string(), 3, 2),
            ("C".to_string(), 5, 1),
            ("D".to_string(), 6, 4),
        ])
    );
}

#[test]
fn test_cycle_detection() {
    let mut scheduler = TaskScheduler::new();
    scheduler.add_task("A", vec!["B"], 1);
    scheduler.add_task("B", vec!["C"], 1);
    scheduler.add_task("C", vec!["A"], 1);

    let schedule = scheduler.schedule_tasks();
    assert_eq!(schedule, Err(ScheduleError::CycleDetected));
}
