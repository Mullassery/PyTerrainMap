# Data Integrity & Resilience

## Core Principle: Append-Only, Immutable Storage

PyTerrainMap is **append-only** and **immutable**. This is fundamental to system resilience.

### What This Means

**Even if a bot goes rogue and adds nonsensical data:**
- ❌ Cannot delete historical observations
- ❌ Cannot modify existing data  
- ❌ Cannot corrupt the historical record
- ✅ All old, good data remains accessible
- ✅ Rogue data appears as NEW observations (adds, doesn't replace)
- ✅ Other bots can still reference clean historical data

### Architecture

```
PyTerrainMap: Append-Only Log

Timeline:

T=1: Bot_A adds temp reading (22°C) at Location_X
     ✅ STORED
     
T=2: Bot_B adds lidar scan at Location_X
     ✅ STORED
     
T=3: Bot_C (rogue) adds garbage: "purple elephants at Location_X"
     ✅ STORED (but labeled as Bot_C, time T=3)
     
T=4: Bot_D queries Location_X history
     ├─ Sees Bot_A's data (22°C, timestamp T=1)
     ├─ Sees Bot_B's data (lidar, timestamp T=2)  
     ├─ Sees Bot_C's garbage (purple elephants, timestamp T=3)
     └─ PyTerrainAI filters out Bot_C's nonsense
         → Bot_D gets: T=1 temp + T=2 lidar (good data preserved)

T=5: Investigation
     ├─ Audit trail shows Bot_C added garbage at T=3
     ├─ Bot_C's credentials revoked
     ├─ Historical truth (T=1, T=2 data) unaffected
```

---

## Data Immutability Guarantees

### What Cannot Happen
```
❌ Rogue bot CANNOT delete Bot_A's observation
❌ Rogue bot CANNOT modify Bot_B's confidence scores
❌ Rogue bot CANNOT alter timestamps of old data
❌ Rogue bot CANNOT change sensor_type of existing observations
❌ Rogue bot CANNOT rewrite history
```

### What CAN Happen
```
✅ Rogue bot CAN add false observations (new entries)
✅ Rogue bot CAN claim to be at wrong location
✅ Rogue bot CAN report wildly wrong values
✅ Rogue bot CAN fabricate sensor readings
```

### How We Handle It

**PyTerrainMap:**
- Stores everything as-is
- Maintains immutable log
- Records source (robot_id, timestamp)

**PyTerrainAI:**
- Detects anomalies (statistical outliers)
- Filters rogue data (doesn't return in context)
- Alerts on suspicious patterns
- Maintains data quality scores
- Still serves old, validated data

---

## Example: Security Breach Scenario

### Scenario: Compromised Bot

```
Bot "security_1" goes rogue at T=100

T=95:  security_1: "All clear at Building_A" ✅ (good)
T=98:  security_1: "Motion detected at exit" ✅ (good)
T=100: COMPROMISED
T=100: security_1: "The sky is purple" ❌ (nonsense)
T=101: security_1: "2+2=5" ❌ (nonsense)
T=102: security_1: "Aliens detected" ❌ (nonsense)

T=110: Other security bots query history
       
PyTerrainMap returns ALL data (including garbage)
       ↓
PyTerrainAI:
├─ Detects T=95, T=98 data is consistent with expected patterns
├─ Detects T=100, T=101, T=102 are statistical anomalies
├─ Returns to bot: T=95 + T=98 data only (filters anomalies)
├─ Alert: "security_1 behavior anomaly detected"
└─ Revoke security_1 credentials

Result:
✅ Other bots still get good historical data
✅ Rogue data identified but not deleted (audit trail intact)
✅ Investigation shows exactly when/what went wrong
✅ System continues operating with clean context
```

---

## Why This Matters

### 1. Tamper-Resistance
- Even if an attacker compromises a robot, they can't erase evidence
- Historical truth is preserved
- Audit trail is tamper-proof

### 2. Data Recovery
- If you discover bad data later, historical good data still exists
- Don't need to "roll back" (append-only prevents that problem)
- Clean observations remain available

### 3. Forensics & Investigation
```
When security_1 goes rogue:
  ✅ We see exactly when behavior changed
  ✅ We can compare T=95 data (good) to T=100 data (bad)
  ✅ We understand the attack timeline
  ✅ We can validate other robots weren't compromised
```

### 4. Collaborative Trust
- Bots can't corrupt each other's historical data
- Multi-bot missions remain trustworthy
- Even if one bot fails, others' data is safe

### 5. Integrity Validation (Critical)
- **Old data = trusted baseline** for validating new data
- New observations compared against historical patterns
- Rogue data detected by deviation from baseline
- Example: "Temperature has always been 20-25°C at this location, new reading of 500°C is flagged as rogue"

---

## Old Data as Validation Baseline

Old, trusted observations serve as the ground truth for flagging suspicious new data:

```
Historical Baseline (from PyTerrainMap):
├─ Location X: Temperature range 20-25°C (100 observations over 30 days)
├─ Location X: Occupancy 5-10 people (consistent pattern)
├─ Location X: No obstacles (clear for 6 months)
└─ Confidence: High (consensus across multiple bots)

New Observation (rogue bot):
├─ Location X: Temperature 500°C
├─ Comparison: Deviates 200x from historical range
├─ Flag: ANOMALY - temperature spike with no infrastructure changes
└─ Action: Filter from context, alert admin

PyTerrainAI workflow:
1. Query PyTerrainMap for historical data (last 30 days)
   → Get trusted baseline
2. Analyze latest 5 observations
   → Compare to baseline
3. Detect deviations
   → Flag new data if significant deviation
4. Return only data consistent with baseline
   → Filtered context to requesting bot
```

### Benefits of This Approach

| Aspect | Benefit |
|--------|---------|
| **Rogue detection** | New data flagged by comparison to historical norm |
| **Attack timeline** | Exactly when behavior changed from normal |
| **Data recovery** | If bot compromised, old data still trustworthy |
| **Continuous validation** | Every new observation validated against history |
| **No data loss** | Both good and bad data preserved (for investigation) |

---

## Implementation Details

### Storage Implications

```rust
// Immutable observation storage
pub struct Observation {
    pub id: Uuid,  // Unique, never changes
    pub robot_id: String,  // Recorded at store time
    pub timestamp: i64,  // Original timestamp, never modified
    pub location: GeoPoint,  // Never changed
    pub value: SensorValue,  // Never changed
    pub confidence: f32,  // Original confidence, never decayed
    pub created_at: i64,  // When stored in map
    pub _immutable: marker::PhantomData,  // Compiler-level immutability
}

// Storage: append-only
impl PyTerrainMap {
    pub async fn push_observation(&self, obs: Observation) -> Result<()> {
        // ADD observation to log
        self.observations.append(obs)?;
        // NEVER UPDATE or DELETE existing observations
        Ok(())
    }
    
    pub async fn query(&self, ...) -> Result<Vec<Observation>> {
        // Return observations as stored (no modifications)
        self.observations.range_query(...)
    }
}

// NO update_observation method
// NO delete_observation method
// NO modify_observation method
```

### Database Schema (if using SQL)

```sql
-- Observations table: append-only
CREATE TABLE observations (
    id UUID PRIMARY KEY,
    robot_id VARCHAR(256) NOT NULL,
    timestamp BIGINT NOT NULL,
    location_lat FLOAT NOT NULL,
    location_lon FLOAT NOT NULL,
    elevation_asl FLOAT,
    sensor_type VARCHAR(256) NOT NULL,
    value JSONB NOT NULL,
    confidence FLOAT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT now(),
    
    -- CRITICAL: No update triggers, no delete triggers
    -- Only INSERT allowed
);

-- Audit log (also immutable)
CREATE TABLE audit_log (
    id UUID PRIMARY KEY,
    bot_id VARCHAR(256) NOT NULL,
    action VARCHAR(256) NOT NULL,  -- "query", "push", "failed_auth"
    location_lat FLOAT,
    location_lon FLOAT,
    timestamp BIGINT NOT NULL,
    success BOOLEAN NOT NULL,
    
    -- For investigating attacks
    created_at BIGINT NOT NULL DEFAULT now(),
);
```

---

## PyTerrainAI's Role: Quality Gates

Even with append-only storage, PyTerrainAI protects data quality:

```python
class DataQualityGate:
    """
    Validates new observations against historical baseline.
    CRITICAL: Anomalies are NOT errors—they may be genuine changes.
    Mark for verification, don't delete.
    """
    
    async def process_observations(self, observations: List[Observation]) -> List[ObservationWithStatus]:
        """
        PyTerrainMap returns ALL observations.
        PyTerrainAI classifies each by comparing against historical baseline:
        - VERIFIED: Consistent with baseline, known patterns
        - ANOMALY_NEEDS_VERIFICATION: Different from baseline, might be real change
        - ROGUE: Consistent pattern of malicious data
        - SENSOR_FAULT: Single bot diverges from own history (likely hardware issue)
        """
        processed = []
        
        for obs in observations:
            # Get historical baseline
            baseline = await self.get_historical_baseline(
                location=obs.location,
                sensor_type=obs.sensor_type,
                days_back=30
            )
            
            # Analyze deviation from baseline
            z_score = self.compute_z_score(obs, baseline)
            deviation_percent = abs(obs.value - baseline.mean) / (baseline.mean + 1e-6) * 100
            
            # CRITICAL DISTINCTION: Anomaly ≠ Error
            if z_score > 3.0:  # Significant deviation
                
                # Is this a CHANGE or a MALFUNCTION?
                if await self.is_rogue_bot(obs.robot_id):
                    # Consistent pattern of malice
                    status = ObservationStatus.ROGUE_BOT
                elif await self.is_sensor_fault(obs.robot_id, obs.sensor_type):
                    # This bot's sensor is broken
                    status = ObservationStatus.SENSOR_FAULT
                else:
                    # GENUINE CHANGE - mark for verification, DON'T FILTER
                    status = ObservationStatus.ANOMALY_NEEDS_VERIFICATION
                    await self.alert_operator(
                        f"Change detected at {obs.location}: "
                        f"{obs.sensor_type} changed {deviation_percent:.1f}%. "
                        f"Send verification bot to confirm."
                    )
            else:
                status = ObservationStatus.VERIFIED
            
            # IMPORTANT: Keep ALL observations, mark with status
            processed.append(ObservationWithStatus(
                observation=obs,
                status=status,
                z_score=z_score,
                deviation_percent=deviation_percent,
                needs_verification=(status == ObservationStatus.ANOMALY_NEEDS_VERIFICATION),
            ))
        
        return processed
    
    async def get_historical_baseline(self, location, sensor_type, days_back=30):
        """
        Query PyTerrainMap for historical observations.
        Use old data to establish baseline for new validation.
        """
        baseline_obs = await self.map_service.query(
            location=location,
            sensor_type=sensor_type,
            days_back=days_back,
            only_high_confidence=True  # Use only trusted observations
        )
        
        # Compute baseline statistics (mean, std, range)
        return BaselineStatistics(
            mean=np.mean([o.value for o in baseline_obs]),
            std=np.std([o.value for o in baseline_obs]),
            min=np.min([o.value for o in baseline_obs]),
            max=np.max([o.value for o in baseline_obs]),
            observation_count=len(baseline_obs),
        )
```

---

## System Properties

### Immutability
- ✅ No data is ever deleted
- ✅ No data is ever modified
- ✅ Every observation has immutable timestamp, source, value

### Append-Only
- ✅ New observations added to end of log
- ✅ No retroactive changes to history
- ✅ Rogue data is NEW entries, not corruption of old

### Auditability
- ✅ Every bot action logged with timestamp
- ✅ Exactly when data was added
- ✅ Exactly what was added and by whom
- ✅ Attack timeline reconstructible

### Resilience to Compromise
- ✅ If one bot compromised, others' data safe
- ✅ Historical truth preserved
- ✅ Forensics enabled
- ✅ Continued operation with filtered context

---

## Edge Cases Handled

### Case 1: Bot Adds Garbage, Then Fixed
```
T=100: Bot_A (compromised): "Purple elephants"
T=110: Bot_A fixed, resumes normal operation: "22°C at location X"

Result:
- Both observations stored
- PyTerrainAI filters out T=100 (anomaly)
- Returns T=110 (valid)
- Audit shows Bot_A recovery
```

### Case 2: Multiple Bots, One Rogue
```
Bot_A (good): "22°C"     ✅
Bot_B (rogue): "99°C"    ❌ (outlier)
Bot_C (good): "21°C"     ✅

PyTerrainAI: "Consensus is 22°C (Bot_B is outlier, filtered)"
```

### Case 3: Historical Data Predates Compromise
```
T=50: Bot_D: "Building intact" (before compromise)
T=100: Bot_D compromised
T=105: Bot_D: "Building exploded"  (rogue)

Query for historical state:
- T=50 data still accessible, still good
- T=105 data flagged as suspect
- Forensics show exact compromise timeline
```

---

## Anomalies ≠ Errors: Detecting Genuine Changes

**Critical distinction:** A flagged anomaly is not necessarily wrong—it may indicate a **genuine change in the environment**.

### Scenario: Temperature Spike (Not an Error)

```
Historical Baseline (30 days):
  Temperature at Location_X: 20-24°C (stable)
  
Day 31 - Bot_A reports:
  Temperature: 45°C
  Flag: ANOMALY (deviation from baseline)
  
Reaction (OLD, WRONG):
  ❌ "Must be sensor error, filter it out"
  ❌ Data lost
  
Reaction (NEW, CORRECT):
  ✅ "Change detected. Mark as unverified. Alert for verification."
  ✅ Data preserved with "NEEDS_VERIFICATION" tag
  
Day 32 - Inspection Bot arrives with camera:
  ├─ Camera confirms: New HVAC unit installed
  ├─ Observation: "HVAC installation at Location_X"
  ├─ Verification: Temperature spike IS legitimate
  └─ Update: Baseline changes (new normal is 40-50°C)
  
Result:
  ✅ Anomaly verified as real change
  ✅ Old data still in map (with context)
  ✅ New baseline established
  ✅ Next bots know: Location_X runs hot now
```

---

## Sensor Malfunctions (Different from Real Changes)

Even honest bots can collect questionable data due to sensor failures:

### Types of Sensor Issues

```
1. Calibration Drift
   └─ Sensor slowly drifts from accurate reading
      Before: 22°C (correct)
      After: 25°C (drifted, sensor needs recalibration)

2. Sudden Failure
   └─ Sensor abruptly fails (physical damage, short circuit)
      Observation: 999°C (nonsensical)
      Cause: Sensor failure, not bot malice

3. Environmental Sensitivity
   └─ Sensor affected by interference
      Thermal: Affected by sun exposure
      LiDAR: Affected by dust/humidity
      USB: RF interference

4. Firmware Bug
   └─ Software issue in sensor driver
      Example: Sensor off-by-factor-of-10
      Robot is honest, sensor code is buggy

5. Power Issues
   └─ Low battery affecting sensor readings
      Observation: Wildly fluctuating values
      Cause: Power supply voltage low
```

### Detection vs Mitigation

```
PyTerrainAI must distinguish:

Rogue Bot:
├─ Intentional bad data
├─ Consistent pattern of nonsense
├─ Alert: "Bot compromised"
└─ Action: Revoke credentials

Sensor Malfunction:
├─ Honest bot, failed sensor
├─ Deviation from bot's historical norm
├─ Alert: "Sensor recalibration needed"
└─ Action: Flag data, suggest diagnostic
```

### Quality Gate Enhancement

```python
class SensorHealthMonitor:
    """Detect and report sensor malfunctions"""
    
    async def validate_sensor_health(self, obs: Observation) -> SensorStatus:
        """
        Compare observation to:
        1. Historical baseline for this bot's sensor
        2. Other bots' readings at same location
        3. Expected sensor accuracy range
        """
        
        # Get this bot's historical sensor data
        bot_history = await self.map_service.query(
            robot_id=obs.robot_id,
            sensor_type=obs.sensor_type,
            days_back=30
        )
        
        # Compute bot's typical accuracy/variance
        bot_baseline = BaselineStatistics(bot_history)
        
        # Compare new observation
        deviation = (obs.value - bot_baseline.mean) / (bot_baseline.std + 1e-6)
        
        if deviation > 5.0:  # 5 sigma = extremely unlikely
            # Distinguish: Bot malice vs sensor failure?
            
            if self.is_consistent_malice(obs.robot_id):
                # Pattern of intentional bad data
                return SensorStatus.ROGUE_BOT
            else:
                # First major deviation from this bot
                return SensorStatus.SENSOR_MALFUNCTION
        
        return SensorStatus.HEALTHY
    
    async def cross_validate_with_peers(self, obs: Observation):
        """
        Compare against other bots' observations at same location.
        Helps distinguish sensor failure vs rogue behavior.
        """
        peer_obs = await self.map_service.query(
            location=obs.location,
            sensor_type=obs.sensor_type,
            exclude_robot=obs.robot_id,
            days_back=1  # Recent peer data
        )
        
        if peer_obs:
            peer_mean = np.mean([o.value for o in peer_obs])
            
            # Does this bot's observation match peers?
            if abs(obs.value - peer_mean) < threshold:
                return "Sensor OK (matches peers)"
            else:
                return "Sensor mismatch (diverges from peers)"
        
        return "No peer data to compare"
```

### Handling Different Observation Statuses

Never assume anomaly = error. Always classify and track.

```python
class AnomalyHandler:
    """
    Process anomalies with verification tracking.
    Anomalies are kept, classified, and verified through subsequent visits.
    """
    
    async def process_observation(self, obs_with_status: ObservationWithStatus) -> None:
        """
        Route observation to appropriate handler based on status.
        """
        if obs_with_status.status == ObservationStatus.VERIFIED:
            # Consistent with baseline - store normally
            await self.store_verified(obs_with_status.observation)
        
        elif obs_with_status.status == ObservationStatus.ANOMALY_NEEDS_VERIFICATION:
            # GENUINE CHANGE DETECTED - Keep it, mark for verification
            await self.store_anomaly_pending_verification(obs_with_status.observation)
            
            # Create verification task
            await self.create_verification_task(
                location=obs_with_status.observation.location,
                reason=f"Temperature anomaly: {obs_with_status.deviation_percent:.1f}% change",
                suggested_sensors=[SensorType.Camera, SensorType.Thermal]
            )
            
            # Alert operator
            await self.alert_operator(
                f"Anomaly at {obs_with_status.observation.location}: "
                f"Needs verification. Sending inspector."
            )
        
        elif obs_with_status.status == ObservationStatus.SENSOR_FAULT:
            # Likely hardware failure - reduce confidence
            obs = obs_with_status.observation
            obs.confidence *= 0.2  # Mark as low-confidence, not deleted
            obs.annotation = "SENSOR_FAULT_SUSPECTED"
            await self.store_anomaly_pending_verification(obs)
            
            # Alert bot owner
            await self.alert_bot(obs.robot_id,
                message=f"Sensor anomaly detected. {obs.sensor_type} readings unreliable. "
                        f"Please recalibrate or replace sensor.")
        
        elif obs_with_status.status == ObservationStatus.ROGUE_BOT:
            # Consistent malicious pattern - revoke
            await self.revoke_bot(obs_with_status.observation.robot_id)
            # Still store (for audit trail), but mark as rogue
            obs = obs_with_status.observation
            obs.annotation = "ROGUE_BOT_DATA"
            await self.store_verified(obs)
```

### Verification Workflow

Anomalies get verified through subsequent bot visits:

```
Step 1: Anomaly Detected
  T=100: Bot_A (thermal): Temperature 45°C (was 22°C baseline)
  Status: ANOMALY_NEEDS_VERIFICATION
  Alert: "Change detected at Location_X"

Step 2: Verification Task Created
  ├─ Location: Location_X
  ├─ Required sensors: Camera (visual verification)
  ├─ Priority: High
  └─ Assigned: Next available inspection bot

Step 3: Inspector Bot Visits
  T=110: Bot_B (camera + thermal) visits Location_X
  ├─ Camera: Confirms HVAC unit newly installed
  ├─ Thermal: Confirms 45°C is new normal (unit generating heat)
  └─ Verdict: ANOMALY_VERIFIED_AS_REAL_CHANGE

Step 4: Baseline Updated
  ├─ Old baseline (invalid): 20-24°C
  ├─ New baseline (current): 40-50°C
  ├─ Change reason: "HVAC unit installed"
  └─ Next bots use new baseline for comparison

Step 5: Context Returned to Bots
  When next bot queries Location_X:
  ├─ Gets both: old observations (20-24°C) and new (40-50°C)
  ├─ Knows: "Environment changed on Day 31"
  ├─ Baseline: Now 40-50°C (not 20-24°C)
  └─ Confidence: High (verified by multiple sensors)
```

### Anomaly Tracking Data Structure

```python
@dataclass
class VerificationStatus:
    """Track anomaly from detection through verification"""
    observation_id: UUID
    location: GeoPoint
    timestamp: int
    anomaly_z_score: float
    anomaly_percent: float
    
    # Verification tracking
    status: AnomalyStatus  # PENDING, VERIFIED_REAL, SENSOR_FAULT, ROGUE
    verification_bot_id: Optional[str] = None
    verification_timestamp: Optional[int] = None
    verification_method: Optional[str] = None  # "camera", "consensus", "human"
    verification_notes: str = ""
    
    # Context for decision-making
    baseline_before: float
    baseline_after: Optional[float] = None
    change_reason: Optional[str] = None
```

---

## Baselines Are Temporal Windows, Not Permanent Truths

**Critical:** Baselines change as environments change. Don't treat them as immutable.

```
Temporal Baseline Evolution:

Period 1 (Days 1-30):
  Location_X baseline: 20-24°C
  ├─ Status: "Normal state"
  ├─ Data: 100 observations from 5 bots
  └─ Confidence: HIGH

Period 2 (Day 31 - Anomaly Detected):
  New observation: 45°C
  ├─ Status: "Anomalous vs Period 1 baseline"
  ├─ Alert: "Change detected"
  └─ Verification initiated

Period 2 (Days 31-35 - Verification):
  Bot_B (camera): "HVAC installed, explains 45°C"
  Bot_C (thermal): "Confirms 42-48°C range"
  Bot_D (thermal): "Confirms 43-47°C range"
  └─ Consensus: 40-50°C is new normal

Period 2 (Days 31-60):
  Location_X baseline: 40-50°C
  ├─ Status: "New normal (environment changed)"
  ├─ Data: Multiple observations confirming
  ├─ Context: "HVAC unit installed on Day 31"
  └─ Confidence: HIGH

Period 3 (Day 61 - Change Again):
  New observation: 15°C (HVAC turned off)
  ├─ Status: "Anomalous vs Period 2 baseline"
  ├─ Alert: "Change detected again"
  └─ Verification process repeats

Key Insight:
├─ Baselines = "what was normal in this time window"
├─ Baselines change when environments change
├─ Multiple observations establish new baseline
├─ Don't assume baseline is permanent
└─ Track baseline evolution over time
```

### Baseline Confidence Increases with Agreement

```
Anomaly Detection Chain:

Day 31, Bot_A reports 45°C:
  Status: NEEDS_VERIFICATION
  Confidence in anomaly: Medium
  
Day 32, Bot_B (independent) reports 44°C:
  Status: VERIFIED_REAL_CHANGE
  Confidence in anomaly: High
  New baseline forming: 40-50°C
  
Day 33, Bot_C reports 46°C:
  Status: CONSENSUS_REACHED
  Confidence in new baseline: VERY HIGH
  Update: Lock in new baseline for Period 2
  
Result:
  ├─ Anomaly is now verified real change
  ├─ New baseline established by consensus
  ├─ All future comparisons use 40-50°C
  └─ Ready for next anomaly detection
```

### Baseline Versioning

```python
@dataclass
class BaselineVersion:
    """Track baseline evolution over time"""
    baseline_id: UUID
    location: GeoPoint
    sensor_type: SensorType
    
    # Temporal window
    start_timestamp: int
    end_timestamp: Optional[int]  # None if current
    
    # Statistical summary
    mean: float
    std: float
    min: float
    max: float
    observation_count: int
    num_unique_bots: int
    
    # Verification
    status: BaselineStatus  # INITIAL, VERIFIED, SUPERSEDED
    confidence: float  # Increases with observations
    
    # Context
    change_reason: Optional[str]  # "HVAC installed", "equipment removed"
    change_verified_by: Optional[str]  # "camera", "consensus", "manual"
```

---

## Summary: Data Integrity Strategy

| Scenario | Detection | Response |
|----------|-----------|----------|
| **Real Environmental Change** | Anomalous vs old baseline, verified by multiple bots | Flag, verify with camera/additional sensors, establish new baseline |
| **Rogue Bot** | Consistent pattern of intentional bad data, doesn't match peers | Mark as rogue, filter from context, but keep in audit trail |
| **Sensor Malfunction** | Single bot deviates, contradicts peer observations | Flag as sensor fault, alert bot owner, reduce confidence |
| **Calibration Drift** | Gradual deviation over time from bot's own history | Alert bot for recalibration, mark observation confidence lower |
| **Transient Glitch** | Single outlier, bot returns to normal next read | Flag, track, include with low confidence (might be verification artifact) |
| **Baseline Evolution** | Multiple bots converge on new value range | Update baseline, track transition period, adjust anomaly detection threshold |

---

## Conclusion

PyTerrainMap's append-only, immutable design ensures that **even if robots go rogue or sensors fail, the historical truth is preserved**. PyTerrainAI's quality gates ensure that **contaminated data doesn't reach operational bots**, while maintaining the complete audit trail for investigation.

Critical insight: **Old data is the validation baseline for new data.** By comparing new observations against 30 days of history, we detect both rogue behavior and sensor malfunctions, distinguishing between intentional attacks and honest equipment failures.

This is why separation of concerns (PyTerrainMap ≠ PyTerrainAI) is critical: storage is pure and immutable, intelligence layer is smart and adaptive.
