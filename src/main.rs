use std::collections::VecDeque;
use std::rc::Rc;
use std::cell::RefCell;

use rand::{thread_rng, Rng};

struct Cpu {
    id: usize,
    clock: u64,
    runq: VecDeque<Rc<RefCell<Task>>>,
    running_task: Rc<RefCell<Task>>,
    idle_task: Rc<RefCell<Task>>
}

impl Cpu {
    fn new(id: usize) -> Self {
        let idle_task_impl = IdleTask {};
        let idle_task = Task::new(0, Box::new(idle_task_impl));
        let idle_task_ref = Rc::new(RefCell::new(idle_task));

        Cpu {
            id: id,
            clock: 0,
            runq: VecDeque::new(),
            running_task: idle_task_ref.clone(),
            idle_task: idle_task_ref.clone(),
        }
    }

    fn add_task(&mut self, task: Rc<RefCell<Task>>) {
        self.runq.push_back(task.clone());
    }

    fn next_task(&mut self) {
        let old_task = self.running_task.clone();
        if old_task.borrow().id != self.idle_task.borrow().id {
            self.runq.push_back(old_task.clone());
        }

        let new_task = self.runq.pop_front();
        let new_task_or_idle = new_task.unwrap_or(self.idle_task.clone());
        self.running_task = new_task_or_idle;
        self.running_task.borrow_mut().state = TaskState::RUNNING;

        let task_slice_output = self.running_task.borrow_mut().run(self);
        self.clock += task_slice_output.clock_consumed;
        match task_slice_output.next_state {
            TaskState::RUNNING => panic!("next state should be RUNNABLE or WAIT"),
            _ => self.running_task.borrow_mut().state = task_slice_output.next_state
        }
    }
}

#[derive(PartialEq)]
enum TaskState {
    RUNNABLE,
    RUNNING,
    WAIT
}

trait TaskImpl {
    fn name(&self) -> &str;

    // returns cpu time consumed
    fn do_work(&mut self) -> u64;
}

struct TaskSliceOutput {
    next_state: TaskState,
    clock_consumed: u64
}

struct Task {
    id: u64,
    state: TaskState,
    total_runtime: u64,
    task_impl: Box<dyn TaskImpl>,
}

impl Task {
    fn new(id: u64, task_impl: Box<dyn TaskImpl>) -> Self {
        Task {
            id: id,
            state: TaskState::RUNNABLE,
            total_runtime: 0,
            task_impl: task_impl
        }
    }

    fn run(&mut self, cpu: &Cpu) -> TaskSliceOutput {
        println!("task {} ({}) running on cpu {}, total runtime {}", self.id, self.task_impl.name(), cpu.id, self.total_runtime);
        let work_quantity = self.task_impl.do_work();
        self.total_runtime += work_quantity;
        TaskSliceOutput {
            next_state: TaskState::RUNNABLE,
            clock_consumed: work_quantity
        }
    }
}

struct IdleTask;
impl TaskImpl for IdleTask {
    fn name(&self) -> &str {
        return "idle";
    }

    fn do_work(&mut self) -> u64 {
        return 1;
    }
}

struct RandomUserTask;
impl TaskImpl for RandomUserTask {
    fn name(&self) -> &str {
        return "randomuser";
    }

    fn do_work(&mut self) -> u64 {
        let mut rng = thread_rng();
        let quantity: u64 = rng.gen_range(1..1000) as u64;
        return quantity;
    }
}


struct Scheduler {
    cpus: Vec<Cpu>
}

impl Scheduler {
    fn new() -> Self {
        Scheduler {
            cpus: Vec::new(),
        }
    }

    fn add_cpus(&mut self, cpu_count: usize) {
        for cpu_id in 0..cpu_count {
            let cpu = Cpu::new(cpu_id);
            self.cpus.push(cpu);
        }
    }

    fn add_tasks(&mut self, task_count: u64) {
        let first_task_id = 1000;

        let cpu_count = self.cpus.len();

        for task_id in first_task_id..first_task_id+task_count {
            let task_impl = RandomUserTask {};
            let task = Rc::new(RefCell::new(Task::new(task_id, Box::new(task_impl))));
            let cpu_id = (task_id % (cpu_count as u64)) as usize;

            self.cpus[cpu_id].add_task(task);
        }
    }

    fn run_forever(&mut self) {
        let mut i = 0;
        loop {
            if i == 1000 {
                break;
            }

            for cpu in self.cpus.iter_mut() {
                cpu.next_task();
                i += 1;
            }
        }
    }

    fn print_cpu_clocks(&self) {
        for cpu in self.cpus.iter() {
            println!("cpu {} has clock {}", cpu.id, cpu.clock);
            println!("  idle time {}", cpu.idle_task.borrow().total_runtime);
        }
    }

    fn print_task_runtime(&self) {
        for cpu in self.cpus.iter() {
            for task in cpu.runq.iter() {
                println!("task {} has total runtime {}", task.borrow().id, task.borrow().total_runtime);
            }
        }
    }
}

fn main() {
    let mut scheduler = Scheduler::new();
    scheduler.add_cpus(8);
    scheduler.add_tasks(64);

    println!("cpu0 tasks:");
    for task in scheduler.cpus[0].runq.iter() {
        println!("task id {}", task.borrow().id);
    }

    scheduler.run_forever();

    println!("###");

    scheduler.print_cpu_clocks();
    scheduler.print_task_runtime();
}
