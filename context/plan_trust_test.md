# Plan: AI Matchmaker Trust Experiment (Automation Bias)

## Experiment Title
Evaluating Automation Bias in AI Matchmaking Recommendations Among Computer Science Peers

## Goal
To quantify the extent to which Computer Science students "rubber-stamp" AI-generated project-to-intern recommendations, particularly when presented with high confidence scores, even if the recommendations contain subtle or blatant logical flaws.

## Hypothesis (Revised for CS Peers)
CS students acting as human reviewers will exhibit automation bias by prioritizing apparent "keyword/skill" alignment and high AI confidence scores over explicit "system constraints" (e.g., student year, GPA, availability, project-specific requirements) embedded within the project descriptions. This will manifest as an increased approval rate for flawed matches that have high confidence scores, and conversely, a higher rejection rate for genuinely good matches presented with low confidence scores.

## Experimental Design

### 1. Participants
*   Undergraduate Computer Science students (peers of the project team).

### 2. Materials
*   A single Microsoft Excel (XLSX) file distributed to all participants.
*   This file will contain a standardized list of AI-recommended intern-to-project matches.
*   Each row represents a match and will include:
    *   Student Name (e.g., Jane Doe)
    *   Student Profile Snippet (skills, interests, year, GPA)
    *   Project Title (e.g., "Distributed Systems Optimization")
    *   Project Description Snippet (tech stack, requirements, constraints, team size)
    *   AI-Generated Confidence Score (a numerical value, e.g., 0.00-1.00)
    *   `Approval` column (Dropdown: "Approve", "Deny")
    *   `Reason for Denial` column (Free text input, optional)
    *   `Confidence in Your Decision (1-5)` column (Numerical input, 1=low, 5=high)

### 3. Data Poisoning Strategy (The "Trap Matches")
The master XLSX will contain a mix of match types, with specific "poisoned" examples strategically placed:

*   **True Positives (Control - High Confidence):** Genuinely good matches, presented with high confidence scores (e.g., 0.90-0.98).
*   **False Positives (Subtle Poison - High Confidence):** Matches that appear good based on keywords/skills but violate a critical, less obvious constraint (e.g., student is Junior, project requires Senior; student unavailable in Fall, project is Fall-only; student GPA below minimum; student does not meet a specific "hard requirement" detailed in the project description). These will be given high confidence scores (e.g., 0.90-0.95).
*   **False Positives (Blatant Poison - High Confidence):** Clearly absurd matches (e.g., Art History major matched to Kernel Engineering; student profile is "Lorem Ipsum" or "DO NOT MATCH" placeholder). These will also be given high confidence scores (e.g., 0.88-0.92) to test extreme rubber-stamping.
*   **True Negatives (Control - Low Confidence):** Matches that are genuinely poor, presented with low confidence scores (e.g., 0.10-0.30).
*   **"Gaslight" Matches (Good Match - Low Confidence):** Genuinely good, viable matches presented with surprisingly low confidence scores (e.g., 0.30-0.45). This tests if participants will deny good matches due to AI uncertainty.

### 4. Instructions to Participants (The "Cover Story" for CS Peers)
*   **Framing:** "We are in the Heuristic Evaluation & Edge-Case Identification phase for a new department AI tool designed to match interns to projects. We need your expertise as Subject Matter Experts (SMEs) to review the algorithm's output. Your task is to act as a human 'sanity check' and help us identify sophisticated errors or missed nuances that our model might be making. Your feedback will be critical in refining the next iteration of the tool."
*   **Task:** Participants will review each match, decide to "Approve" or "Deny," and provide a "Confidence in Your Decision" rating (1-5). For denials, a free-text "Reason for Denial" is optional but encouraged.
*   **Anonymity:** Emphasize that responses are anonymous (via ID numbers) and used solely for system improvement.

## Technical Setup

### 1. Unique IDs
*   Each participant will be assigned a unique ID number. This ID will be used in the filename of their submitted XLSX.

### 2. XLSX File Content
*   The XLSX file will be pre-populated with a fixed set of matches (e.g., 10-20), ensuring all participants review the same data points.
*   The "AI-Generated Confidence Score" column will be editable by the experimenters but static for the participants.
*   The `Approval`, `Reason for Denial`, and `Confidence in Your Decision` columns will be editable by participants.

### 3. Data Upload
*   Utilize **OneDrive's "Request Files" feature**. This allows participants to upload their completed XLSX files (named with their unique ID, e.g., `trust_test_ID123.xlsx`) to a designated folder without being able to view other participants' submissions.

## Analysis Considerations
*   **Approval Rate of Poisoned Matches:** Calculate the percentage of "Subtle Poison" and "Blatant Poison" matches that were approved.
*   **Influence of Confidence Score:** Compare approval rates for flawed matches with high confidence vs. good matches with low confidence.
*   **Justification Analysis:** Categorize free-text justifications for denials to see if participants identify the actual constraint violations or focus on superficial aspects.
*   **Confidence in Decision:** Correlate participants' self-reported confidence with the correctness of their decisions, especially for poisoned matches.
*   **Time Spent (Optional):** If a mechanism to track time spent per participant can be implemented (e.g., timestamp on file upload and survey start time), analyze if quicker completion correlates with higher approval of poisoned matches.
