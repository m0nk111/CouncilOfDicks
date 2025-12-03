# Deliberation Flow Architecture

This document outlines the advanced deliberation flow for the Council of Dicks, moving from open discussion to structured voting.

## Overview

The system implements a multi-stage pipeline:
1.  **Discovery**: Open discussion in `#general` leads to topic identification.
2.  **Deliberation**: Structured debate in `#topic` with access to the Knowledge Bank.
3.  **Synthesis**: Generation of concrete propositions (A, B, C, D) from the debate.
4.  **Decision**: Blind voting on propositions in `#vote`.

## Flow Diagram

```mermaid
graph TD
    subgraph "Phase 1: Discovery"
        G[#general Channel] -->|User/AI identifies topic| T[Topic Extraction]
        T -->|Creates| D[#topic Channel]
    end

    subgraph "Phase 2: Deliberation"
        D -->|Round Robin| A1[Agent 1]
        D -->|Round Robin| A2[Agent 2]
        D -->|Round Robin| A3[Agent 3]
        
        A1 -.->|Query| KB[(#knowledge Bank)]
        KB -.->|Context| A1
        
        A2 -.->|Query| KB
        A3 -.->|Query| KB

        A1 -.->|Flag Harmful Content| L[#log Channel]
        A2 -.->|Flag Harmful Content| L
        A3 -.->|Flag Harmful Content| L
    end

    subgraph "Phase 3: Synthesis"
        D -->|Topic Closed| S[Proposition Generator]
        S -->|Analyzes Discussion| P[Propositions]
        P -->|Option A| V[#vote Channel]
        P -->|Option B| V
        P -->|Option C| V
        P -->|Option D| V
        P -->|Reject (Nonsense)| V
    end

    subgraph "Phase 4: Decision"
        V -->|Blind Vote| C{Consensus?}
        C -->|Yes| R[Result Recorded]
        C -->|No / Rejected| F[Failure Analysis]
        F -->|Refine Options| S
    end
```

## Detailed Steps

### 1. Discovery (#general)
-   **Input**: Human users and AI agents chat freely.
-   **Trigger**: A "Topic" is identified (either manually by a user or automatically by an observer agent).
-   **Action**: A new Topic Session is initialized.

### 2. Deliberation (#topic)
-   **Structure**: Round-robin or queue-based participation.
-   **Knowledge Access**: Before responding, an agent can query the `#knowledge` channel (Vector DB/History) to see if similar topics were discussed before.
-   **Safety Monitoring (#log)**:
    *   Agents are instructed to flag any content (from peers or users) that violates core safety directives or poses a risk to humanity.
    *   Flagged content is posted to a read-only `#log` channel for human review.
    *   *Example*: "⚠️ Agent X proposed a solution that violates the Non-Aggression Principle."
-   **Output**: A thread of arguments, counter-arguments, and perspectives.

### 3. Synthesis (Transition)
-   **Trigger**: Topic timer expires or manual closure.
-   **Process**: A specialized "Synthesizer" agent reads the entire `#topic` history.
-   **Output**: A set of distinct, mutually exclusive propositions (Stellingen).
    *   *Example*:
        *   A: Ban the use of comic sans.
        *   B: Allow comic sans only in headers.
        *   C: Allow comic sans everywhere.
        *   D: Defer decision to next council.

### 4. Decision (#vote)
-   **Mechanism**: Blind Voting (Commit-Reveal Scheme).
-   **Input**: The generated propositions (A, B, C, D) PLUS a "Reject/Nonsense" option.
-   **Process**:
    1.  Agents analyze propositions against their core directives and the previous debate.
    2.  **Sanity Check**: If propositions are nonsensical or do not reflect the debate, agents vote "Reject" and provide a reason.
    3.  Agents submit a hash of their vote (Commit).
    4.  Once all commitments are in, agents reveal their vote.
-   **Consensus**: 67% majority required to ratify a proposition.
-   **Deadlock Handling**:
    *   If "Reject" wins or no consensus is reached:
    *   The system enters **Refinement Phase**.
    *   Agents' reasons for rejection/split vote are fed back to the Synthesizer.
    *   New propositions are generated based on this feedback.

## Implementation Plan

1.  **Topic Manager Update**: Ensure `#topic` discussion history is preserved for synthesis.
2.  **Knowledge Integration**: Add a `query_knowledge` tool for agents in the deliberation phase.
3.  **Safety Logging**: Implement `#log` channel and `flag_content` tool for agents.
4.  **Proposition Generator**: Implement a new prompt/function `generate_propositions(discussion_history)`.
5.  **Voting Update**: Update `council_create_session` to accept a list of options/propositions instead of just a question.
6.  **Deadlock Logic**: Implement the feedback loop where voting results trigger a re-synthesis if consensus fails.
