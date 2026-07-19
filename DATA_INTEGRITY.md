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
    """Filters out suspicious observations before returning to bot"""
    
    async def filter_observations(self, observations: List[Observation]) -> List[Observation]:
        """
        PyTerrainMap returns ALL observations (good + bad).
        PyTerrainAI filters bad ones based on mission context.
        """
        filtered = []
        
        for obs in observations:
            # Check 1: Is this observation plausible?
            if not self.is_plausible(obs):
                self.alert(f"Implausible observation from {obs.robot_id}: {obs.value}")
                continue  # Filter out
            
            # Check 2: Does this contradict recent good data?
            if self.contradicts_baseline(obs):
                self.flag_as_suspect(obs)
                # Might include with low confidence, or exclude entirely
            
            # Check 3: Is the source reliable?
            reliability = self.robot_reliability(obs.robot_id)
            if reliability < 0.5:  # Compromised
                self.alert(f"Bot {obs.robot_id} has low reliability")
                continue  # Filter out
            
            filtered.append(obs)
        
        return filtered
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

## Conclusion

PyTerrainMap's append-only, immutable design ensures that **even if robots go rogue, the historical truth is preserved**. PyTerrainAI's quality gates ensure that **contaminated data doesn't reach operational bots**, while maintaining the complete audit trail for investigation.

This is why separation of concerns (PyTerrainMap ≠ PyTerrainAI) is critical: storage is pure and immutable, intelligence layer is smart and adaptive.
