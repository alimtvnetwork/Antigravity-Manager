//! IP Security Module Tests
//! Comprehensive test suite for IP security monitoring capabilities
//!
//! Test objectives:
//! 1. Verify the correctness of the IP black/white list function
//! 2. Verify CIDR matching logic
//! 3. Verify expiration time processing
//! 4. Verification does not affect the performance of the main process
//! 5. Verify the atomicity and consistency of database operations

#[cfg(test)]
mod ip_filter_middleware_tests;
#[cfg(test)]
mod performance_benchmarks;
#[cfg(test)]
mod security_db_tests;
