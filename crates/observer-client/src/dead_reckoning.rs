//! Dead reckoning module for predictive agent position interpolation.
//!
//! This module implements client-side prediction to compensate for network latency
//! and provide smooth agent movement visualization between authoritative state updates.
//!
//! # Algorithm
//!
//! Uses kinematic equations for motion prediction:
//! - Position: `p(t) = p₀ + v₀·Δt + ½·a·Δt²`
//! - Velocity: `v(t) = v₀ + a·Δt`
//!
//! When a new authoritative state arrives, we blend the predicted position with
//! the actual position to avoid jarring corrections ("rubber-banding").

use glam::Vec3;
use std::time::{Duration, Instant};

/// Predicted agent state using dead reckoning.
///
/// Maintains both the predicted state (calculated locally) and the last
/// authoritative state (received from the network) for interpolation.
#[derive(Debug, Clone)]
pub struct PredictedAgent {
    /// Last authoritative position from network
    pub authoritative_position: Vec3,

    /// Last authoritative velocity from network
    pub authoritative_velocity: Vec3,

    /// Last authoritative acceleration from network
    pub authoritative_acceleration: Vec3,

    /// Timestamp when authoritative state was received
    pub last_update: Instant,

    /// Current predicted position (calculated)
    pub predicted_position: Vec3,

    /// Current predicted velocity (calculated)
    pub predicted_velocity: Vec3,
}

impl PredictedAgent {
    /// Create a new predicted agent from authoritative state.
    pub fn new(position: Vec3, velocity: Vec3, acceleration: Vec3) -> Self {
        Self {
            authoritative_position: position,
            authoritative_velocity: velocity,
            authoritative_acceleration: acceleration,
            last_update: Instant::now(),
            predicted_position: position,
            predicted_velocity: velocity,
        }
    }

    /// Update with new authoritative state from the network.
    ///
    /// This resets the prediction baseline and blends the predicted position
    /// with the new authoritative position to avoid sudden jumps.
    pub fn update_authoritative(
        &mut self,
        position: Vec3,
        velocity: Vec3,
        acceleration: Vec3,
        blend_factor: f32,
    ) {
        // Blend predicted position with new authoritative position
        // blend_factor = 0.0 -> keep predicted (smooth but potentially wrong)
        // blend_factor = 1.0 -> snap to authoritative (accurate but jarring)
        // blend_factor = 0.3 -> good balance for most cases
        self.predicted_position = self
            .predicted_position
            .lerp(position, blend_factor.clamp(0.0, 1.0));

        // Update authoritative state
        self.authoritative_position = position;
        self.authoritative_velocity = velocity;
        self.authoritative_acceleration = acceleration;
        self.last_update = Instant::now();

        // Reset predicted velocity to match authoritative
        self.predicted_velocity = velocity;
    }

    /// Predict the current position based on elapsed time since last update.
    ///
    /// Uses kinematic equations:
    /// - position = p₀ + v₀·Δt + ½·a·Δt²
    /// - velocity = v₀ + a·Δt
    pub fn predict(&mut self) -> Vec3 {
        let dt = self.last_update.elapsed().as_secs_f32();

        // Kinematic prediction
        self.predicted_position = self.authoritative_position
            + self.authoritative_velocity * dt
            + 0.5 * self.authoritative_acceleration * dt * dt;

        self.predicted_velocity = self.authoritative_velocity + self.authoritative_acceleration * dt;

        self.predicted_position
    }

    /// Get the current display position with exponential smoothing.
    ///
    /// This provides an additional layer of smoothing on top of the
    /// kinematic prediction to reduce jitter from frame-to-frame.
    pub fn get_smoothed_position(&self, alpha: f32) -> Vec3 {
        // Exponential moving average for smooth display
        // alpha = 0.0 -> no smoothing (use predicted directly)
        // alpha = 0.1 -> heavy smoothing (slow to respond)
        // alpha = 0.3 -> moderate smoothing (good balance)
        self.predicted_position
            .lerp(self.authoritative_position, alpha.clamp(0.0, 1.0))
    }

    /// Calculate prediction error (distance between predicted and authoritative).
    ///
    /// Useful for debugging and adaptive correction strength tuning.
    pub fn prediction_error(&self) -> f32 {
        self.predicted_position
            .distance(self.authoritative_position)
    }

    /// Check if prediction error exceeds threshold (indicates desync).
    pub fn needs_correction(&self, max_error: f32) -> bool {
        self.prediction_error() > max_error
    }
}

/// Dead reckoning engine managing predictions for multiple agents.
///
/// This is the main interface for the observer client to maintain
/// smooth agent visualizations despite network latency.
pub struct DeadReckoning {
    /// Blend factor for position corrections (0.0 to 1.0)
    blend_factor: f32,

    /// Smoothing alpha for exponential moving average (0.0 to 1.0)
    smoothing_alpha: f32,

    /// Maximum allowed prediction error before forcing correction (units)
    max_error_threshold: f32,
}

impl DeadReckoning {
    /// Create a new dead reckoning engine with default parameters.
    pub fn new() -> Self {
        Self {
            blend_factor: 0.3,        // 30% blend toward authoritative
            smoothing_alpha: 0.2,     // 20% smoothing
            max_error_threshold: 10.0, // Force correction at 10 units error
        }
    }

    /// Create a new dead reckoning engine with custom parameters.
    pub fn with_params(blend_factor: f32, smoothing_alpha: f32, max_error_threshold: f32) -> Self {
        Self {
            blend_factor: blend_factor.clamp(0.0, 1.0),
            smoothing_alpha: smoothing_alpha.clamp(0.0, 1.0),
            max_error_threshold: max_error_threshold.max(0.0),
        }
    }

    /// Create a predicted agent state from initial values.
    pub fn create_agent(&self, position: Vec3, velocity: Vec3, acceleration: Vec3) -> PredictedAgent {
        PredictedAgent::new(position, velocity, acceleration)
    }

    /// Update an agent with new authoritative state from network.
    pub fn update_agent(
        &self,
        agent: &mut PredictedAgent,
        position: Vec3,
        velocity: Vec3,
        acceleration: Vec3,
    ) {
        // Use stronger correction if error is too large
        let blend = if agent.needs_correction(self.max_error_threshold) {
            1.0 // Force snap to authoritative
        } else {
            self.blend_factor
        };

        agent.update_authoritative(position, velocity, acceleration, blend);
    }

    /// Predict current position for an agent.
    pub fn predict_position(&self, agent: &mut PredictedAgent) -> Vec3 {
        agent.predict()
    }

    /// Get smoothed display position for an agent.
    pub fn get_display_position(&self, agent: &PredictedAgent) -> Vec3 {
        agent.get_smoothed_position(self.smoothing_alpha)
    }

    /// Calculate time since last authoritative update for an agent.
    pub fn time_since_update(&self, agent: &PredictedAgent) -> Duration {
        agent.last_update.elapsed()
    }

    /// Adjust blend factor dynamically (for runtime tuning).
    pub fn set_blend_factor(&mut self, factor: f32) {
        self.blend_factor = factor.clamp(0.0, 1.0);
    }

    /// Adjust smoothing alpha dynamically (for runtime tuning).
    pub fn set_smoothing_alpha(&mut self, alpha: f32) {
        self.smoothing_alpha = alpha.clamp(0.0, 1.0);
    }

    /// Adjust error threshold dynamically (for runtime tuning).
    pub fn set_error_threshold(&mut self, threshold: f32) {
        self.max_error_threshold = threshold.max(0.0);
    }
}

impl Default for DeadReckoning {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: Implement projective velocity blending for smoother interpolation
// This would predict future velocity changes based on historical acceleration patterns,
// reducing overshoot when agents change direction suddenly.

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_dead_reckoning_creation() {
        let dr = DeadReckoning::new();
        assert_eq!(dr.blend_factor, 0.3);
        assert_eq!(dr.smoothing_alpha, 0.2);
        assert_eq!(dr.max_error_threshold, 10.0);
    }

    #[test]
    fn test_predicted_agent_initialization() {
        let position = Vec3::new(1.0, 2.0, 3.0);
        let velocity = Vec3::new(0.5, 0.0, -0.5);
        let acceleration = Vec3::ZERO;

        let agent = PredictedAgent::new(position, velocity, acceleration);

        assert_eq!(agent.authoritative_position, position);
        assert_eq!(agent.predicted_position, position);
        assert_eq!(agent.authoritative_velocity, velocity);
    }

    #[test]
    fn test_kinematic_prediction() {
        let position = Vec3::new(0.0, 0.0, 0.0);
        let velocity = Vec3::new(1.0, 0.0, 0.0); // Moving 1 unit/sec in X
        let acceleration = Vec3::ZERO;

        let mut agent = PredictedAgent::new(position, velocity, acceleration);

        // Simulate 100ms delay
        thread::sleep(std::time::Duration::from_millis(100));

        let predicted = agent.predict();

        // Should have moved ~0.1 units in X direction
        assert!(predicted.x > 0.05 && predicted.x < 0.15);
        assert_eq!(predicted.y, 0.0);
        assert_eq!(predicted.z, 0.0);
    }

    #[test]
    fn test_acceleration_prediction() {
        let position = Vec3::new(0.0, 0.0, 0.0);
        let velocity = Vec3::ZERO;
        let acceleration = Vec3::new(2.0, 0.0, 0.0); // 2 units/sec² in X

        let mut agent = PredictedAgent::new(position, velocity, acceleration);

        // Simulate 100ms delay
        thread::sleep(std::time::Duration::from_millis(100));

        let predicted = agent.predict();

        // position = 0.5 * a * t² = 0.5 * 2.0 * 0.1² = 0.01
        assert!(predicted.x > 0.005 && predicted.x < 0.015);

        // velocity = a * t = 2.0 * 0.1 = 0.2
        assert!(agent.predicted_velocity.x > 0.15 && agent.predicted_velocity.x < 0.25);
    }

    #[test]
    fn test_authoritative_update_blending() {
        let mut agent = PredictedAgent::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::ZERO,
            Vec3::ZERO,
        );

        // Set predicted position to somewhere else
        agent.predicted_position = Vec3::new(5.0, 0.0, 0.0);

        // Update with new authoritative position
        let new_pos = Vec3::new(10.0, 0.0, 0.0);
        agent.update_authoritative(new_pos, Vec3::ZERO, Vec3::ZERO, 0.5);

        // With 0.5 blend, should be halfway between predicted (5) and authoritative (10)
        assert!((agent.predicted_position.x - 7.5).abs() < 0.1);
    }

    #[test]
    fn test_prediction_error_calculation() {
        let mut agent = PredictedAgent::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::ZERO,
            Vec3::ZERO,
        );

        agent.predicted_position = Vec3::new(3.0, 4.0, 0.0);

        // Error should be distance: sqrt(3² + 4²) = 5.0
        let error = agent.prediction_error();
        assert!((error - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_needs_correction() {
        let mut agent = PredictedAgent::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::ZERO,
            Vec3::ZERO,
        );

        // Small error - no correction needed
        agent.predicted_position = Vec3::new(1.0, 0.0, 0.0);
        assert!(!agent.needs_correction(5.0));

        // Large error - correction needed
        agent.predicted_position = Vec3::new(10.0, 0.0, 0.0);
        assert!(agent.needs_correction(5.0));
    }

    #[test]
    fn test_dead_reckoning_parameter_clamping() {
        let dr = DeadReckoning::with_params(1.5, -0.5, -10.0);

        // Values should be clamped to valid ranges
        assert_eq!(dr.blend_factor, 1.0); // Clamped from 1.5
        assert_eq!(dr.smoothing_alpha, 0.0); // Clamped from -0.5
        assert_eq!(dr.max_error_threshold, 0.0); // Clamped from -10.0
    }
}
