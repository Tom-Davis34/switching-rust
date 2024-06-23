//! Shared traits and structures for dopri5 and dop853.
use std::{fmt};
use chrono::{Utc, DateTime, DurationRound, Duration};
use nalgebra::OVector;
use thiserror::Error;

pub mod butcher_tableau;
pub mod controller;
pub mod dop853;
mod dopri5;
pub mod foode;
mod rk4;

/// Trait needed to be implemented by the user.
pub trait System<V> {
    /// System of ordinary differential equations.
    fn system(&self, x: f32, y: &V, dy: &mut V);
    /// Stop function called at every successful integration step. The integration is stopped when this function returns true.
    fn solout(&mut self, _x: f32, _y: &V, _dy: &V) -> bool {
        false
    }
}

pub trait Integratable<V>
{
    fn integrate(&mut self) -> Result<Stats, IntegrationError>;
    fn x_out(&self) ->  &Vec<f32>;
    fn _out(&self) -> &Vec<V>;
}



/// Enumeration of the types of the integration output.
#[derive(PartialEq, Eq)]
pub enum OutputType {
    Dense,
    Sparse,
}

/// Enumeration of the errors that may arise during integration.
#[derive(Debug, Error)]
pub enum IntegrationError {
    #[error("Stopped at x = {x}. Need more than {n_step} steps.")]
    MaxNumStepReached { x: f32, n_step: u32 },
    #[error("Stopped at x = {x}. Step size underflow.")]
    StepSizeUnderflow { x: f32 },
    #[error("The problem seems to become stiff at x = {x}.")]
    StiffnessDetected { x: f32 },
}

/// Contains some statistics of the integration.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Stats {
    pub num_eval: u32,
    pub accepted_steps: u32,
    pub rejected_steps: u32,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

impl Stats {
    pub(crate) fn new() -> Stats {
        Stats {
            num_eval: 0,
            accepted_steps: 0,
            rejected_steps: 0,
            start_time: None,
            end_time: None,
        }
    }

    /// Prints some statistics related to the integration process.
    #[deprecated(since = "0.2.0", note = "Use std::fmt::Display instead")]
    pub fn print(&self) {
        println!("{}", self);
    }

    pub fn duration(&self) -> Option<Duration>{
        match self.start_time{
            Some(val) => Some(self.end_time?.signed_duration_since(val)),
            None => None,
        }
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Number of function evaluations: {}", self.num_eval)?;
        writeln!(f, "Number of accepted steps: {}", self.accepted_steps)?;
        writeln!(f, "Number of rejected steps: {}", self.rejected_steps)?;
        match self.start_time{
            Some(val) =>  writeln!(f, "Start Time: {}", val)?,
            None => writeln!(f, "Start Time: None")?,
        }

        match self.duration(){
            Some(val) =>  writeln!(f, "Duration: {}", val),
            None => writeln!(f, "Duration: Not finished"),
        }
    }
}
