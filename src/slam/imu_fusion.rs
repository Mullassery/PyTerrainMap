//! IMU and Visual Odometry Fusion using Extended Kalman Filter
//!
//! Combines inertial measurements (accelerometer, gyroscope) with visual odometry
//! to estimate robot pose and velocity with reduced drift.

use crate::types::{Result, Error};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// IMU state (bias, noise parameters)
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct IMUState {
    /// Gyroscope bias (rad/s)
    pub gyro_bias: (f32, f32, f32),
    /// Accelerometer bias (m/s²)
    pub accel_bias: (f32, f32, f32),
    /// Gyroscope noise (rad/s)
    pub gyro_noise: f32,
    /// Accelerometer noise (m/s²)
    pub accel_noise: f32,
}

impl IMUState {
    /// Create default IMU state
    pub fn new() -> Self {
        IMUState {
            gyro_bias: (0.0, 0.0, 0.0),
            accel_bias: (0.0, 0.0, 0.0),
            gyro_noise: 0.001,     // rad/s
            accel_noise: 0.01,     // m/s²
        }
    }

    /// Update gyroscope bias (exponential moving average)
    pub fn update_gyro_bias(&mut self, measured: (f32, f32, f32), alpha: f32) {
        self.gyro_bias.0 = self.gyro_bias.0 * (1.0 - alpha) + measured.0 * alpha;
        self.gyro_bias.1 = self.gyro_bias.1 * (1.0 - alpha) + measured.1 * alpha;
        self.gyro_bias.2 = self.gyro_bias.2 * (1.0 - alpha) + measured.2 * alpha;
    }
}

impl Default for IMUState {
    fn default() -> Self {
        Self::new()
    }
}

/// IMU preintegration (integrates measurements between two keyframes)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IMUPreintegration {
    /// Integrated rotation (quaternion)
    pub delta_rotation: (f32, f32, f32, f32), // (qx, qy, qz, qw)
    /// Integrated velocity change (m/s)
    pub delta_velocity: (f32, f32, f32),
    /// Integrated position change (m)
    pub delta_position: (f32, f32, f32),
    /// Covariance of rotation (3x3 diagonal)
    pub rotation_covariance: [f32; 3],
    /// Covariance of velocity (3x3 diagonal)
    pub velocity_covariance: [f32; 3],
    /// Covariance of position (3x3 diagonal)
    pub position_covariance: [f32; 3],
    /// Jacobian w.r.t. gyro bias
    pub jacobian_gyro_bias: [[f32; 3]; 3],
    /// Jacobian w.r.t. accel bias
    pub jacobian_accel_bias: [[f32; 3]; 3],
    /// Number of measurements integrated
    pub measurement_count: u32,
    /// Total integration time (seconds)
    pub integration_time: f32,
}

impl IMUPreintegration {
    /// Create new preintegration (reset at keyframe)
    pub fn new() -> Self {
        IMUPreintegration {
            delta_rotation: (0.0, 0.0, 0.0, 1.0), // Identity quaternion
            delta_velocity: (0.0, 0.0, 0.0),
            delta_position: (0.0, 0.0, 0.0),
            rotation_covariance: [0.001, 0.001, 0.001],
            velocity_covariance: [0.01, 0.01, 0.01],
            position_covariance: [0.1, 0.1, 0.1],
            jacobian_gyro_bias: [[0.0; 3]; 3],
            jacobian_accel_bias: [[0.0; 3]; 3],
            measurement_count: 0,
            integration_time: 0.0,
        }
    }

    /// Add IMU measurement to preintegration
    pub fn add_measurement(
        &mut self,
        accel: (f32, f32, f32),
        gyro: (f32, f32, f32),
        dt: f32,
        imu_state: &IMUState,
    ) {
        // Remove bias from measurements
        let accel_corrected = (
            accel.0 - imu_state.accel_bias.0,
            accel.1 - imu_state.accel_bias.1,
            accel.2 - imu_state.accel_bias.2,
        );
        let gyro_corrected = (
            gyro.0 - imu_state.gyro_bias.0,
            gyro.1 - imu_state.gyro_bias.1,
            gyro.2 - imu_state.gyro_bias.2,
        );

        // Update preintegration: position += velocity * dt + 0.5 * accel * dt²
        let half_dt_sq = 0.5 * dt * dt;
        self.delta_position.0 += self.delta_velocity.0 * dt + accel_corrected.0 * half_dt_sq;
        self.delta_position.1 += self.delta_velocity.1 * dt + accel_corrected.1 * half_dt_sq;
        self.delta_position.2 += self.delta_velocity.2 * dt + accel_corrected.2 * half_dt_sq;

        // Update velocity: velocity += accel * dt
        self.delta_velocity.0 += accel_corrected.0 * dt;
        self.delta_velocity.1 += accel_corrected.1 * dt;
        self.delta_velocity.2 += accel_corrected.2 * dt;

        // Update rotation (simple integration: theta = integral(omega * dt))
        let angle = (gyro_corrected.0 * gyro_corrected.0
            + gyro_corrected.1 * gyro_corrected.1
            + gyro_corrected.2 * gyro_corrected.2)
            .sqrt()
            * dt;

        if angle > 1e-6 {
            let sin_half = (angle / 2.0).sin();
            let cos_half = (angle / 2.0).cos();
            let dq = (
                (gyro_corrected.0 / angle) * sin_half,
                (gyro_corrected.1 / angle) * sin_half,
                (gyro_corrected.2 / angle) * sin_half,
                cos_half,
            );
            // Quaternion multiplication: q = q * dq
            self.delta_rotation = quaternion_multiply(self.delta_rotation, dq);
        }

        // Update covariances
        self.rotation_covariance[0] += imu_state.gyro_noise * imu_state.gyro_noise * dt;
        self.rotation_covariance[1] += imu_state.gyro_noise * imu_state.gyro_noise * dt;
        self.rotation_covariance[2] += imu_state.gyro_noise * imu_state.gyro_noise * dt;

        self.measurement_count += 1;
        self.integration_time += dt;
    }

    /// Reset preintegration
    pub fn reset(&mut self) {
        self.delta_rotation = (0.0, 0.0, 0.0, 1.0);
        self.delta_velocity = (0.0, 0.0, 0.0);
        self.delta_position = (0.0, 0.0, 0.0);
        self.measurement_count = 0;
        self.integration_time = 0.0;
    }
}

impl Default for IMUPreintegration {
    fn default() -> Self {
        Self::new()
    }
}

/// IMU and visual odometry fusion using Extended Kalman Filter
pub struct IMUFusion {
    /// Current IMU state
    pub imu_state: IMUState,
    /// Current preintegration buffer
    pub preintegration: IMUPreintegration,
    /// Filter state: [x, y, z, vx, vy, vz] (position and velocity)
    pub state: [f32; 6],
    /// State covariance (6x6 diagonal)
    pub covariance: [f32; 6],
    /// Process noise
    pub process_noise: f32,
    /// Measurement noise (visual odometry)
    pub measurement_noise: f32,
}

impl IMUFusion {
    /// Create IMU fusion filter
    pub fn new() -> Self {
        IMUFusion {
            imu_state: IMUState::new(),
            preintegration: IMUPreintegration::new(),
            state: [0.0; 6], // Start at origin with zero velocity
            covariance: [0.1, 0.1, 0.1, 0.01, 0.01, 0.01], // Initial uncertainty
            process_noise: 0.001,
            measurement_noise: 0.01,
        }
    }

    /// Predict state using preintegrated IMU measurements
    pub fn predict(&mut self, dt: f32) {
        // Constant velocity model (without acceleration, since we're using preintegrated IMU)
        let vel_x = self.state[3];
        let vel_y = self.state[4];
        let vel_z = self.state[5];

        // Update position
        self.state[0] += vel_x * dt;
        self.state[1] += vel_y * dt;
        self.state[2] += vel_z * dt;

        // Update covariance (grows with time)
        for i in 0..6 {
            self.covariance[i] += self.process_noise * dt;
        }
    }

    /// Update state with visual odometry measurement
    pub fn update_visual_odometry(&mut self, delta_pos: (f32, f32, f32), delta_vel: (f32, f32, f32)) {
        // Measurement: observed position and velocity change
        let innovation_pos = [
            delta_pos.0 - (self.state[0] + self.state[3] * 0.1),
            delta_pos.1 - (self.state[1] + self.state[4] * 0.1),
            delta_pos.2 - (self.state[2] + self.state[5] * 0.1),
        ];

        // Kalman gain (simplified: K = P * H^T / (H * P * H^T + R))
        let innovation_variance = self.covariance[0] + self.measurement_noise;
        let kalman_gain = self.covariance[0] / innovation_variance;

        // Update state
        self.state[0] += kalman_gain * innovation_pos[0];
        self.state[1] += kalman_gain * innovation_pos[1];
        self.state[2] += kalman_gain * innovation_pos[2];

        // Update velocity from measurement
        self.state[3] = 0.9 * self.state[3] + 0.1 * delta_vel.0;
        self.state[4] = 0.9 * self.state[4] + 0.1 * delta_vel.1;
        self.state[5] = 0.9 * self.state[5] + 0.1 * delta_vel.2;

        // Update covariance (reduction due to measurement)
        for i in 0..6 {
            self.covariance[i] *= (1.0 - kalman_gain);
        }
    }

    /// Update gyroscope bias from zero-velocity update (ZUPT)
    pub fn zupt_update(&mut self, gyro: (f32, f32, f32)) {
        // When robot is stationary, gyro should be zero
        // Update bias estimate (fast adaptation)
        self.imu_state.update_gyro_bias(gyro, 0.1);
    }

    /// Get current position
    pub fn get_position(&self) -> (f32, f32, f32) {
        (self.state[0], self.state[1], self.state[2])
    }

    /// Get current velocity
    pub fn get_velocity(&self) -> (f32, f32, f32) {
        (self.state[3], self.state[4], self.state[5])
    }

    /// Get position uncertainty
    pub fn get_position_uncertainty(&self) -> (f32, f32, f32) {
        (self.covariance[0].sqrt(), self.covariance[1].sqrt(), self.covariance[2].sqrt())
    }
}

impl Default for IMUFusion {
    fn default() -> Self {
        Self::new()
    }
}

/// Quaternion multiplication (q1 * q2)
fn quaternion_multiply(q1: (f32, f32, f32, f32), q2: (f32, f32, f32, f32)) -> (f32, f32, f32, f32) {
    let (x1, y1, z1, w1) = q1;
    let (x2, y2, z2, w2) = q2;

    (
        w1 * x2 + x1 * w2 + y1 * z2 - z1 * y2,
        w1 * y2 - x1 * z2 + y1 * w2 + z1 * x2,
        w1 * z2 + x1 * y2 - y1 * x2 + z1 * w2,
        w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imu_state_creation() {
        let state = IMUState::new();
        assert_eq!(state.gyro_bias, (0.0, 0.0, 0.0));
        assert_eq!(state.accel_bias, (0.0, 0.0, 0.0));
    }

    #[test]
    fn test_preintegration_add_measurement() {
        let mut preint = IMUPreintegration::new();
        let imu_state = IMUState::new();

        preint.add_measurement((0.0, 0.0, 9.81), (0.0, 0.0, 0.0), 0.01, &imu_state);
        assert_eq!(preint.measurement_count, 1);
        assert!(preint.integration_time > 0.0);
    }

    #[test]
    fn test_imu_fusion_predict() {
        let mut fusion = IMUFusion::new();
        fusion.state = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0]; // 1 m/s in x direction
        fusion.predict(1.0);
        assert!(fusion.state[0] > 0.0); // Should have moved in x
    }

    #[test]
    fn test_imu_fusion_update() {
        let mut fusion = IMUFusion::new();
        fusion.update_visual_odometry((1.0, 0.0, 0.0), (1.0, 0.0, 0.0));
        let (x, y, z) = fusion.get_position();
        assert!(x > 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(z, 0.0);
    }

    #[test]
    fn test_quaternion_multiply() {
        // q * identity = q
        let q = (0.0, 0.0, 0.0, 1.0);
        let identity = (0.0, 0.0, 0.0, 1.0);
        let result = quaternion_multiply(q, identity);
        assert!((result.0 - q.0).abs() < 1e-5);
        assert!((result.3 - q.3).abs() < 1e-5);
    }
}
