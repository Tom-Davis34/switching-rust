//! Adaptive step size control.

/// Used for adaptive step size control
pub struct Controller {
    alpha: f32,
    beta: f32,
    facc1: f32,
    facc2: f32,
    fac_old: f32,
    h_max: f32,
    reject: bool,
    safety_factor: f32,
    posneg: f32,
}

impl Controller {
    /// Creates a controller responsible for adaptive step size control.
    ///
    /// # Arguments
    ///
    /// * `alpha`   - &#945; coefficient of the PI controller
    /// * `beta`    - &#946; coefficient of the PI controller
    /// * `fac_max` - Maximum factor between two successive steps
    /// * `fac_min` - Minimum factor between two successive steps
    /// * `h_max`   - Maximum step size
    /// * `safety_factor`   - Safety factor of the PI controller
    ///
    pub fn new(
        alpha: f32,
        beta: f32,
        fac_max: f32,
        fac_min: f32,
        h_max: f32,
        safety_factor: f32,
        posneg: f32,
    ) -> Controller {
        Controller {
            alpha,
            beta,
            facc1: 1.0 / fac_min,
            facc2: 1.0 / fac_max,
            fac_old: 1.0E-4,
            h_max: h_max.abs(),
            reject: false,
            safety_factor,
            posneg,
        }
    }

    /// Determines if the step must be accepted or rejected and adapts the step size accordingly.
    pub fn accept(&mut self, err: f32, h: f32, h_new: &mut f32) -> bool {
        let fac11 = err.powf(self.alpha);
        let mut fac = fac11 * self.fac_old.powf(-self.beta);
        fac = (self.facc2).max((self.facc1).min(fac / self.safety_factor));
        *h_new = h / fac;

        if err <= 1.0 {
            // Accept step
            self.fac_old = err.max(1.0E-4);

            if h_new.abs() > self.h_max {
                *h_new = self.posneg * self.h_max;
            }
            if self.reject {
                *h_new = self.posneg * h_new.abs().min(h.abs());
            }

            self.reject = false;
            true
        } else {
            // Reject step
            *h_new = h / ((self.facc1).min(fac11 / self.safety_factor));
            self.reject = true;
            false
        }
    }

    /// Returns the maximum step size allowed.
    pub fn h_max(&self) -> f32 {
        self.h_max
    }
}
