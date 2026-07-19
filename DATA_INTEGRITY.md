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
    Uses old data to flag rogue/compromised data.
    """
    
    async def filter_observations(self, observations: List[Observation]) -> List[Observation]:
        """
        PyTerrainMap returns ALL observations (good + bad).
        PyTerrainAI filters bad ones by comparing against historical baseline.
        """
        filtered = []
        
        for obs in observations:
            # CRITICAL: Check against historical baseline
            baseline = await self.get_historical_baseline(
                location=obs.location,
                sensor_type=obs.sensor_type,
                days_back=30  # 30 days of history
            )
            
            # Check 1: Does this observation match historical patterns?
            if not self.is_consistent_with_baseline(obs, baseline):
                z_score = self.compute_z_score(obs, baseline)
                if z_score > 3.0:  # 99.7% confidence it's anomalous
                    self.alert(f"Rogue observation from {obs.robot_id}: z-score={z_score}")
                    continue  # Filter out - too far from baseline
            
            # Check 2: Is this observation plausible?
            if not self.is_physically_plausible(obs):
                self.alert(f"Implausible observation from {obs.robot_id}: {obs.value}")
                continue  # Filter out
            
            # Check 3: Is the source reliable?
            reliability = self.robot_reliability(obs.robot_id)
            if reliability < 0.5:  # Compromised
                self.alert(f"Bot {obs.robot_id} has low reliability, flagging new data")
                # Don't filter entirely, but mark with low confidence
                obs.confidence *= (1.0 - (0.5 - reliability))  # Reduce confidence
            
            filtered.append(obs)
        
        return filtered
    
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

## Sensor Malfunctions (Different from Rogue Bots)

Even honest bots can collect bad data due to sensor failures:

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

### Handling Sensor Malfunctions

Instead of filtering out all anomalous data:

1. **Detect the malfunction** (statistical deviation)
2. **Classify it** (malice vs hardware failure)
3. **Alert the bot** ("Thermal sensor needs recalibration")
4. **Reduce confidence** (mark obs as low-confidence, not deleted)
5. **Use peer data** (other bots' observations if available)
6. **Log for investigation** (later diagnostics can review)

```python
class MalfunctionHandling:
    """Handle sensor failures gracefully"""
    
    async def process_suspicious_observation(self, obs: Observation) -> Observation:
        """
        Don't blindly filter - classify and handle intelligently
        """
        status = await self.validate_sensor_health(obs)
        
        if status == SensorStatus.HEALTHY:
            return obs  # Return as-is
        
        elif status == SensorStatus.SENSOR_MALFUNCTION:
            # Alert bot to investigate
            await self.alert_bot(obs.robot_id, 
                message="Thermal sensor readings appear anomalous. Recalibration recommended.")
            
            # Mark with low confidence, but keep in map
            obs.confidence *= 0.3  # Reduce from 0.95 to 0.28
            obs.annotation = "SENSOR_MALFUNCTION_FLAG"
            return obs  # Return with warning flag
        
        elif status == SensorStatus.ROGUE_BOT:
            # Revoke and filter
            await self.revoke_bot(obs.robot_id)
            return None  # Filter out entirely
        
        return obs
```

---

## Summary: Data Integrity Strategy

| Scenario | Detection | Response |
|----------|-----------|----------|
| **Rogue Bot** | Consistent pattern of intentional bad data | Filter + revoke credentials |
| **Sensor Malfunction** | Single bot deviates from own baseline, matches peers | Flag + alert bot + reduce confidence |
| **Calibration Drift** | Gradual deviation over time | Alert for recalibration |
| **Transient Glitch** | Single outlier, bot returns to normal next read | Log + keep (single observation won't skew) |
| **Environmental Interference** | Sensor sensitive to conditions | Document interference source |

---

## Conclusion

PyTerrainMap's append-only, immutable design ensures that **even if robots go rogue or sensors fail, the historical truth is preserved**. PyTerrainAI's quality gates ensure that **contaminated data doesn't reach operational bots**, while maintaining the complete audit trail for investigation.

Critical insight: **Old data is the validation baseline for new data.** By comparing new observations against 30 days of history, we detect both rogue behavior and sensor malfunctions, distinguishing between intentional attacks and honest equipment failures.

This is why separation of concerns (PyTerrainMap ≠ PyTerrainAI) is critical: storage is pure and immutable, intelligence layer is smart and adaptive.
