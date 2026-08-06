//! Query Optimization & Planning
//!
//! Phase 5.2: Intelligent query optimization with cost estimation,
//! plan generation, and adaptive query execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Query execution plan
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub plan_id: u64,
    pub steps: Vec<QueryStep>,
    pub estimated_cost: f32,
    pub estimated_rows: u64,
    pub plan_time_us: i64,
}

impl ExecutionPlan {
    /// Create new execution plan
    pub fn new(plan_id: u64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        ExecutionPlan {
            plan_id,
            steps: Vec::new(),
            estimated_cost: 0.0,
            estimated_rows: 0,
            plan_time_us: now,
        }
    }

    /// Add execution step
    pub fn add_step(&mut self, step: QueryStep) {
        self.estimated_cost += step.estimated_cost;
        self.steps.push(step);
    }

    /// Get plan complexity (0.0-1.0)
    pub fn complexity(&self) -> f32 {
        (self.steps.len() as f32 / 10.0).min(1.0)
    }
}

/// Single query execution step
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryStep {
    pub step_type: StepType,
    pub index: u32,
    pub estimated_cost: f32,
    pub estimated_rows: u64,
}

impl QueryStep {
    /// Create new step
    pub fn new(step_type: StepType, index: u32, cost: f32, rows: u64) -> Self {
        QueryStep {
            step_type,
            index,
            estimated_cost: cost,
            estimated_rows: rows,
        }
    }
}

/// Query step types
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum StepType {
    IndexScan,
    SequentialScan,
    CacheHit,
    CacheMiss,
    TemporalFilter,
    SpatialFilter,
    QualityFilter,
    Merge,
    Sort,
    Aggregate,
}

/// Query statistics for cost estimation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryStatistics {
    pub total_points: u64,
    pub avg_points_per_location: u32,
    pub index_cardinality: u32,
    pub cache_hit_rate: f32,
    pub avg_query_selectivity: f32,
}

impl QueryStatistics {
    /// Create new statistics
    pub fn new() -> Self {
        QueryStatistics {
            total_points: 0,
            avg_points_per_location: 100,
            index_cardinality: 1000,
            cache_hit_rate: 0.5,
            avg_query_selectivity: 0.1,
        }
    }
}

impl Default for QueryStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Cost model for query estimation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CostModel {
    pub index_scan_cost_per_row: f32,
    pub seq_scan_cost_per_row: f32,
    pub filter_cost_per_row: f32,
    pub sort_cost_multiplier: f32,
    pub cache_hit_benefit: f32,
}

impl CostModel {
    /// Create default cost model
    pub fn new() -> Self {
        CostModel {
            index_scan_cost_per_row: 0.01,
            seq_scan_cost_per_row: 0.1,
            filter_cost_per_row: 0.001,
            sort_cost_multiplier: 1.5,
            cache_hit_benefit: 0.9, // 90% reduction
        }
    }

    /// Estimate index scan cost
    pub fn index_scan_cost(&self, rows: u64) -> f32 {
        rows as f32 * self.index_scan_cost_per_row
    }

    /// Estimate sequential scan cost
    pub fn seq_scan_cost(&self, rows: u64) -> f32 {
        rows as f32 * self.seq_scan_cost_per_row
    }

    /// Estimate filter cost
    pub fn filter_cost(&self, rows: u64, selectivity: f32) -> f32 {
        let output_rows = rows as f32 * selectivity;
        output_rows * self.filter_cost_per_row
    }

    /// Apply cache benefit
    pub fn apply_cache_benefit(&self, cost: f32, hit_rate: f32) -> f32 {
        cost * (1.0 - (hit_rate * self.cache_hit_benefit))
    }
}

impl Default for CostModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Query optimizer with plan generation
pub struct QueryOptimizer {
    pub cache_enabled: bool,
    pub statistics: QueryStatistics,
    pub cost_model: CostModel,
    pub plans_generated: u64,
    pub plan_history: HashMap<u64, ExecutionPlan>,
}

impl QueryOptimizer {
    /// Create new query optimizer
    pub fn new() -> Self {
        QueryOptimizer {
            cache_enabled: true,
            statistics: QueryStatistics::new(),
            cost_model: CostModel::new(),
            plans_generated: 0,
            plan_history: HashMap::new(),
        }
    }

    /// Generate execution plan for spatial-temporal query
    pub fn plan_spatio_temporal_query(
        &mut self,
        x_range: f32,
        y_range: f32,
        time_range_us: i64,
        quality_threshold: f32,
    ) -> ExecutionPlan {
        let plan_id = self.plans_generated;
        self.plans_generated += 1;

        let mut plan = ExecutionPlan::new(plan_id);

        // Estimate rows at each step
        let spatial_selectivity = (x_range * y_range) / 10000.0; // Normalized to 100x100 area
        let temporal_selectivity = (time_range_us as f32 / 1_000_000.0) / 86400.0; // Normalized to 1 day
        let quality_selectivity = 1.0 - quality_threshold;

        let rows_after_spatial = (self.statistics.total_points as f32 * spatial_selectivity) as u64;
        let rows_after_temporal =
            (rows_after_spatial as f32 * temporal_selectivity).max(1.0) as u64;
        let rows_after_quality =
            (rows_after_temporal as f32 * quality_selectivity).max(1.0) as u64;

        // Step 1: Spatial index scan
        let spatial_cost = self.cost_model.index_scan_cost(rows_after_spatial);
        plan.add_step(QueryStep::new(
            StepType::IndexScan,
            0,
            spatial_cost,
            rows_after_spatial,
        ));

        // Step 2: Check cache
        if self.cache_enabled {
            let cache_cost = spatial_cost * (1.0 - self.statistics.cache_hit_rate);
            plan.add_step(QueryStep::new(
                StepType::CacheHit,
                1,
                cache_cost,
                rows_after_spatial,
            ));
        }

        // Step 3: Temporal filter
        let temporal_cost = self.cost_model.filter_cost(rows_after_spatial, temporal_selectivity);
        plan.add_step(QueryStep::new(
            StepType::TemporalFilter,
            2,
            temporal_cost,
            rows_after_temporal,
        ));

        // Step 4: Quality filter
        let quality_cost = self.cost_model.filter_cost(rows_after_temporal, quality_selectivity);
        plan.add_step(QueryStep::new(
            StepType::QualityFilter,
            3,
            quality_cost,
            rows_after_quality,
        ));

        plan.estimated_rows = rows_after_quality;
        self.plan_history.insert(plan_id, plan.clone());
        plan
    }

    /// Generate plan for nearest neighbor query
    pub fn plan_nearest_neighbor_query(&mut self, k: u32) -> ExecutionPlan {
        let plan_id = self.plans_generated;
        self.plans_generated += 1;

        let mut plan = ExecutionPlan::new(plan_id);

        // For k-NN, we typically scan an index region
        let estimated_scanned = (k as f32 * 10.0) as u64; // Scan ~10x k candidates
        let index_cost = self.cost_model.index_scan_cost(estimated_scanned);

        plan.add_step(QueryStep::new(
            StepType::IndexScan,
            0,
            index_cost,
            estimated_scanned,
        ));

        // Sort to find nearest
        let sort_cost = index_cost * self.cost_model.sort_cost_multiplier;
        plan.add_step(QueryStep::new(
            StepType::Sort,
            1,
            sort_cost,
            k as u64,
        ));

        plan.estimated_rows = k as u64;
        self.plan_history.insert(plan_id, plan.clone());
        plan
    }

    /// Generate plan for aggregation query
    pub fn plan_aggregation_query(&mut self, table_rows: u64) -> ExecutionPlan {
        let plan_id = self.plans_generated;
        self.plans_generated += 1;

        let mut plan = ExecutionPlan::new(plan_id);

        let seq_cost = self.cost_model.seq_scan_cost(table_rows);
        plan.add_step(QueryStep::new(
            StepType::SequentialScan,
            0,
            seq_cost,
            table_rows,
        ));

        let agg_cost = seq_cost * 0.5; // Aggregation is typically cheaper
        plan.add_step(QueryStep::new(
            StepType::Aggregate,
            1,
            agg_cost,
            1,
        ));

        plan.estimated_rows = 1;
        self.plan_history.insert(plan_id, plan.clone());
        plan
    }

    /// Get plan statistics
    pub fn plan_statistics(&self) -> PlanStatistics {
        let total_cost: f32 = self.plan_history.values().map(|p| p.estimated_cost).sum();
        let avg_cost = if self.plan_history.is_empty() {
            0.0
        } else {
            total_cost / self.plan_history.len() as f32
        };

        PlanStatistics {
            total_plans_generated: self.plans_generated,
            cached_plans: self.plan_history.len() as u32,
            total_estimated_cost: total_cost,
            average_plan_cost: avg_cost,
        }
    }
}

impl Default for QueryOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Plan statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanStatistics {
    pub total_plans_generated: u64,
    pub cached_plans: u32,
    pub total_estimated_cost: f32,
    pub average_plan_cost: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_plan() {
        let mut plan = ExecutionPlan::new(1);
        let step = QueryStep::new(StepType::IndexScan, 0, 10.0, 100);
        plan.add_step(step);

        assert_eq!(plan.estimated_rows, 0);
        assert_eq!(plan.steps.len(), 1);
    }

    #[test]
    fn test_cost_model() {
        let model = CostModel::new();
        let cost = model.index_scan_cost(1000);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cost_model_with_cache_benefit() {
        let model = CostModel::new();
        let original_cost = 100.0;
        let reduced_cost = model.apply_cache_benefit(original_cost, 0.9);

        assert!(reduced_cost < original_cost);
    }

    #[test]
    fn test_query_optimizer_spatio_temporal() {
        let mut optimizer = QueryOptimizer::new();
        let plan = optimizer.plan_spatio_temporal_query(10.0, 10.0, 1_000_000, 0.5);

        assert!(plan.estimated_cost > 0.0);
        assert_eq!(plan.steps.len(), 4); // spatial, cache, temporal, quality
    }

    #[test]
    fn test_query_optimizer_nearest_neighbor() {
        let mut optimizer = QueryOptimizer::new();
        let plan = optimizer.plan_nearest_neighbor_query(10);

        assert!(plan.estimated_cost > 0.0);
        assert_eq!(plan.estimated_rows, 10);
    }

    #[test]
    fn test_query_optimizer_aggregation() {
        let mut optimizer = QueryOptimizer::new();
        let plan = optimizer.plan_aggregation_query(10000);

        assert!(plan.estimated_cost > 0.0);
        assert_eq!(plan.estimated_rows, 1);
    }

    #[test]
    fn test_optimizer_statistics() {
        let mut optimizer = QueryOptimizer::new();
        optimizer.plan_spatio_temporal_query(10.0, 10.0, 1_000_000, 0.5);
        optimizer.plan_nearest_neighbor_query(10);

        let stats = optimizer.plan_statistics();
        assert_eq!(stats.total_plans_generated, 2);
        assert!(stats.average_plan_cost > 0.0);
    }
}
