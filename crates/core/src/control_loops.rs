//! Example control loop implementations for real-time scheduling

use crate::scheduler::{ControlLoopTask, SchedulerResult};
use std::time::Instant;

/// A simple control loop that maintains state and timing
#[derive(Debug)]
pub struct ExampleControlLoop {
    name: String,
    iteration: u32,
    state: f32,
    start_time: Instant,
}

impl ExampleControlLoop {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            iteration: 0,
            state: 0.0,
            start_time: Instant::now(),
        }
    }
}

impl ControlLoopTask for ExampleControlLoop {
    fn execute(&mut self) -> SchedulerResult<()> {
        self.iteration += 1;

        // Simulate some control logic
        self.state = (self.iteration as f32 * 0.1).sin();

        // Every 50 iterations (~500ms at 100Hz), print status
        if self.iteration % 50 == 0 {
            let elapsed = self.start_time.elapsed().as_secs_f32();
            println!(
                "[{}] Control: iteration {}, state {:.3}, elapsed {:.2}s",
                self.name, self.iteration, self.state, elapsed
            );
        }

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// A control loop that simulates a PID controller
#[derive(Debug)]
pub struct PidControlLoop {
    name: String,
    setpoint: f32,
    current_value: f32,
    integral: f32,
    last_error: f32,
    kp: f32,
    ki: f32,
    kd: f32,
    iteration: u32,
}

impl PidControlLoop {
    pub fn new(name: impl Into<String>, setpoint: f32) -> Self {
        Self {
            name: name.into(),
            setpoint,
            current_value: 0.0,
            integral: 0.0,
            last_error: 0.0,
            kp: 0.5,
            ki: 0.1,
            kd: 0.2,
            iteration: 0,
        }
    }
}

impl ControlLoopTask for PidControlLoop {
    fn execute(&mut self) -> SchedulerResult<()> {
        self.iteration += 1;

        // Calculate error
        let error = self.setpoint - self.current_value;

        // Proportional term
        let p = self.kp * error;

        // Integral term (accumulate error)
        self.integral += error;
        let i = self.ki * self.integral;

        // Derivative term (rate of change)
        let d = self.kd * (error - self.last_error);

        // Calculate control output
        let output = p + i + d;

        // Clamp output
        let output = output.clamp(-1.0, 1.0);

        // Simulate system response: move toward setpoint
        self.current_value += output * 0.01;

        self.last_error = error;

        // Print every 100 iterations (~1s at 100Hz)
        if self.iteration % 100 == 0 {
            println!(
                "[{}] PID: iteration {}, setpoint {:.2}, current {:.2}, error {:.2}, output {:.2}",
                self.name, self.iteration, self.setpoint, self.current_value, error, output
            );
        }

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
