# Council Of Dicks - Anti-Gaming Mechanisms

## The Problem: Strategic Voting

Without safeguards, council members could:
- **Bandwagon**: Always vote with perceived majority
- **Collusion**: Coordinate votes with other members
- **Vote Trading**: "I vote for you, you vote for me"
- **Prediction Gaming**: Analyze patterns and vote accordingly
- **Sybil Attacks**: Create multiple identities to gain influence

## Defense Mechanisms

### 1. Blind Deliberation Rounds

Council members cannot see each other's votes until after they've committed their own.

```rust
struct DeliberationRound {
    round_number: u32,
    
    // Votes are cryptographically committed before reveal
    committed_votes: HashMap<MemberId, CryptoCommitment>,
    
    // Only revealed after ALL members have committed
    revealed_votes: Option<HashMap<MemberId, Vote>>,
    
    // Prevents late-joiners from seeing patterns
    commitment_deadline: Timestamp,
}

impl DeliberationRound {
    fn commit_vote(&mut self, member: MemberId, vote: Vote) -> Result<()> {
        // Create cryptographic commitment
        let commitment = hash(vote + random_nonce());
        
        self.committed_votes.insert(member, commitment);
        
        // Vote cannot be changed once committed
        Ok(())
    }
    
    fn reveal_votes(&mut self) -> Result<()> {
        // Only after ALL members have committed
        if self.committed_votes.len() == self.expected_members {
            // Verify all commitments match revealed votes
            for (member, vote) in &self.revealed_votes {
                verify_commitment(member, vote)?;
            }
            Ok(())
        } else {
            Err("Not all votes committed yet")
        }
    }
}
```

**Effect:** Members can't see which way the wind is blowing before voting.

---

### 2. Randomized Council Selection

Users don't choose which council members participate - the network does randomly.

```rust
fn select_council(
    question: &Question,
    network: &Network,
    required_size: usize,
) -> Vec<CouncilMember> {
    // Hash of question + current block + random salt
    let seed = hash(question.text + network.current_block() + random_salt());
    
    // Deterministic but unpredictable selection
    let mut rng = SeededRng::from_seed(seed);
    
    // Weight by tier but ensure representation
    let candidates = network.available_members()
        .weighted_by_tier()
        .shuffle(&mut rng);
    
    // Ensure diversity: no more than 40% from same tier
    enforce_diversity_constraints(candidates, required_size)
}

fn enforce_diversity_constraints(
    mut candidates: Vec<CouncilMember>,
    size: usize,
) -> Vec<CouncilMember> {
    let mut selected = Vec::new();
    let mut tier_counts = HashMap::new();
    let max_per_tier = (size as f64 * 0.4).ceil() as usize;
    
    for candidate in candidates {
        let count = tier_counts.entry(candidate.tier).or_insert(0);
        
        if *count < max_per_tier {
            selected.push(candidate);
            *count += 1;
        }
        
        if selected.len() == size {
            break;
        }
    }
    
    selected
}
```

**Effect:** Members can't know in advance who they'll be voting with.

---

### 3. Delayed Reputation Updates

Rankings don't update immediately - preventing real-time feedback loops.

```rust
struct ReputationUpdate {
    member: MemberId,
    decision: DecisionId,
    metrics: Metrics,
    
    // Not applied until delay period expires
    effective_date: Timestamp,
}

const REPUTATION_DELAY: Duration = Duration::days(7);

impl Network {
    fn apply_reputation_updates(&mut self) {
        let now = current_time();
        
        // Only apply updates older than delay period
        for update in self.pending_updates.iter() {
            if update.effective_date <= now {
                self.apply_update(update);
            }
        }
    }
}
```

**Effect:** Members can't immediately see if their gaming strategy "worked".

---

### 4. Retrospective Validation

Past decisions are re-evaluated when outcomes become known.

```rust
struct DecisionValidation {
    decision_id: DecisionId,
    original_verdict: Verdict,
    actual_outcome: Option<Outcome>,
    validation_date: Timestamp,
}

impl Council {
    fn validate_past_decision(&mut self, decision_id: DecisionId, outcome: Outcome) {
        let decision = self.get_decision(decision_id);
        
        // Which members were correct?
        for member in decision.participants {
            if member.vote.aligns_with(outcome) {
                member.retrospective_score += 1.0;
            } else {
                member.retrospective_score -= 0.5;
            }
        }
        
        // If entire council was wrong, no one is penalized
        // (prevents penalizing contrarian thinkers)
        if decision.verdict != outcome && decision.consensus_level > 0.8 {
            // Clear unanimous failure - system issue, not individual
            rollback_penalties(decision);
        }
    }
}
```

**Effect:** Long-term accuracy matters more than short-term appearances.

---

### 5. Anti-Collusion: Vote Independence Scoring

Detect if members consistently vote together (possible collusion).

```rust
struct CollusionDetector {
    vote_pairs: HashMap<(MemberId, MemberId), VoteCorrelation>,
}

struct VoteCorrelation {
    total_shared_councils: u64,
    votes_together: u64,
    independence_score: f64,  // 0.0 = perfect correlation, 1.0 = independent
}

impl CollusionDetector {
    fn analyze_independence(&self, m1: MemberId, m2: MemberId) -> f64 {
        let correlation = self.vote_pairs.get(&(m1, m2)).unwrap();
        
        let agreement_rate = correlation.votes_together as f64 
                           / correlation.total_shared_councils as f64;
        
        // Expected agreement by chance (based on consensus rates)
        let expected = self.expected_agreement(m1, m2);
        
        // Deviation from expected
        let deviation = (agreement_rate - expected).abs();
        
        // High deviation = suspicious
        if deviation > 0.3 {
            flag_for_review(m1, m2);
        }
        
        1.0 - deviation
    }
}
```

**Effect:** Consistently voting together flags members for review.

---

### 6. Minority Opinion Rewards

Sometimes the dissenting voice is the correct one - reward independent thinking.

```rust
fn calculate_dissent_bonus(member: &CouncilMember, decision: &Decision) -> f64 {
    // Did member dissent from consensus?
    if decision.consensus_level > 0.8 && !member.voted_with_majority() {
        
        // Was the dissent well-reasoned?
        let reasoning_quality = evaluate_reasoning(member.argument);
        
        if reasoning_quality > 0.7 {
            // Bonus for thoughtful dissent
            return 5.0 * reasoning_quality;
        }
    }
    
    0.0
}
```

**Effect:** Contrarian thinking is valued, prevents groupthink.

---

### 7. Question Entropy Analysis

Detect if a member only performs well on certain types of questions.

```rust
struct QuestionProfile {
    member: MemberId,
    performance_by_category: HashMap<Category, f64>,
    entropy: f64,  // Low entropy = specialist, High = generalist
}

impl Network {
    fn detect_question_cherry_picking(&self, member: MemberId) -> bool {
        let profile = self.get_question_profile(member);
        
        // If member only excels in narrow categories, flag it
        if profile.entropy < 0.3 {
            // Check if they're declining participation in weak areas
            let participation_rate = self.participation_rate(member);
            
            if participation_rate < 0.5 {
                // Likely cherry-picking questions
                return true;
            }
        }
        
        false
    }
}
```

**Effect:** Members must perform across diverse topics, can't specialize to game rankings.

---

### 8. Byzantine Fault Tolerance

Assume up to 33% of council members may be malicious.

```rust
const MAX_BYZANTINE_RATIO: f64 = 0.33;

fn calculate_consensus_with_bft(votes: &[Vote]) -> Verdict {
    let total = votes.len();
    let min_honest = (total as f64 * (1.0 - MAX_BYZANTINE_RATIO)).ceil() as usize;
    
    // Need >66% agreement to overcome potential Byzantine actors
    let required_majority = (total as f64 * 0.67).ceil() as usize;
    
    for verdict in possible_verdicts(votes) {
        let support = votes.iter().filter(|v| v.supports(verdict)).count();
        
        if support >= required_majority {
            return verdict;
        }
    }
    
    // No supermajority - return split decision
    Verdict::Split(votes)
}
```

**Effect:** Colluding minority (<33%) cannot control outcomes.

---

### 9. Proof of Reasoning

Members must provide detailed reasoning, not just votes.

```rust
struct Vote {
    verdict: Verdict,
    reasoning: String,
    confidence: f64,
    
    // Must reference sources or past decisions
    references: Vec<Reference>,
    
    // Cryptographic proof this reasoning came from this member
    signature: Signature,
}

fn validate_vote(vote: &Vote) -> Result<()> {
    // Minimum reasoning length
    if vote.reasoning.len() < 100 {
        return Err("Insufficient reasoning");
    }
    
    // Must provide evidence or logic
    if vote.references.is_empty() {
        return Err("No supporting references");
    }
    
    // Reasoning must be unique (not copy-pasted)
    if is_duplicate_reasoning(vote.reasoning) {
        return Err("Duplicate reasoning detected");
    }
    
    Ok(())
}
```

**Effect:** Low-effort strategic votes are rejected.

---

### 10. Time-Varied Question Weights

Questions asked during high-traffic periods are weighted less.

```rust
fn calculate_question_weight(timestamp: Timestamp, network: &Network) -> f64 {
    let current_load = network.active_councils_at(timestamp);
    let average_load = network.average_daily_load();
    
    // High load = potentially easier to hide strategic behavior
    let load_ratio = current_load as f64 / average_load as f64;
    
    // Questions during peak times count less toward rankings
    if load_ratio > 2.0 {
        0.5  // Half weight
    } else if load_ratio > 1.5 {
        0.75
    } else {
        1.0  // Full weight
    }
}
```

**Effect:** Gaming during peak times when oversight is low is less rewarding.

---

### 11. Peer Review Audits

Higher-tier members randomly audit lower-tier decisions.

```rust
struct Audit {
    auditor: MemberId,  // Must be higher tier
    decision: DecisionId,
    findings: AuditFindings,
}

impl Network {
    fn schedule_random_audits(&mut self) {
        // 10% of decisions get audited
        for decision in self.recent_decisions.choose_random(0.1) {
            
            // Assign Citadel or Prime member as auditor
            let auditor = self.select_auditor(decision);
            
            // Auditor reviews reasoning quality
            let audit = auditor.review(decision);
            
            if audit.finds_gaming_behavior() {
                penalize_members(decision.participants);
            }
        }
    }
}
```

**Effect:** Random audits create uncertainty - gaming might be caught.

---

## Combined Effect

With all mechanisms active:

```
✅ Votes are blind until all committed
✅ Council selection is unpredictable
✅ Reputation updates are delayed
✅ Past decisions are re-validated
✅ Collusion is detected and flagged
✅ Dissent is rewarded when justified
✅ Cherry-picking is prevented
✅ Byzantine actors can't dominate
✅ Reasoning quality is enforced
✅ Gaming during peak times is discouraged
✅ Random audits catch bad actors
```

**Result:** Gaming the system requires more effort than simply being a good council member.

---

## Constitutional Safeguards

Encoded in the protocol:

```rust
const IMMUTABLE_RULES: &[Rule] = &[
    Rule::VotesAreBlind,
    Rule::SelectionIsRandom,
    Rule::ReasoningRequired,
    Rule::AuditsAreRandom,
    Rule::NoRealtimeReputation,
    Rule::DissentIsValued,
    Rule::ByzantineTolerance,
];

// These cannot be changed without 80% Citadel vote + 60% network vote
```

---

*Make honesty easier than deception.* 🛡️
