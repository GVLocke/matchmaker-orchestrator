# LLM Strategy: Cost & Privacy Analysis (Self-Hosted vs. OpenAI)
**Date:** 2026-02-02

## Executive Summary
**Decision:** Use **OpenAI (API)** for Production.
**Reasoning:** Drastically lower cost for the expected volume, zero operational overhead, and sufficient privacy guarantees via Enterprise agreements.
**Alternative:** Local LLM (Ollama) is recommended for *local development* to save costs and enable offline work.

## 1. Privacy & Redaction
**Question:** *Can we redact PII before sending to OpenAI?*
**Verdict:** **No.**
*   **Technical Catch-22:** Reliable redaction requires an LLM-level understanding of context. If you have a model smart enough to perfectly redact, you already have the parser you're trying to build.
*   **Risk:** Regex/Rule-based redaction is brittle. Missed PII violates the security promise. Aggressive redaction destroys context (e.g., distinguishing "Java" the language from "Java" the location or name).
*   **Solution:** Rely on **Architectural** or **Contractual** security, not sanitization.

## 2. Cost Analysis

### Scenario A: AWS Self-Hosted (EC2)
Hosting a model capable of high-quality extraction (e.g., Llama 3 8B/70B) requires GPU compute.
*   **Instance:** `g5.xlarge` (NVIDIA A10G, 24GB VRAM) or `g4dn.xlarge` (T4, 16GB VRAM).
*   **Hourly Cost:** ~$0.50 - $1.00 (On-Demand).
*   **Monthly Cost:** **$365 - $730** (Running 24/7).
    *   *Note: Auto-scaling (spinning down to 0) adds significant latency (cold starts of 5-10 mins) and complexity.*

### Scenario B: OpenAI API (`gpt-4o-mini`)
*   **Cost per Token:** ~$0.15 / 1M input tokens.
*   **Estimated Tokens per Resume:** ~2,000 (Input) + ~500 (Output).
*   **Cost per Resume:** ~$0.0006.
*   **Monthly Cost:** Variable.

### Break-even Calculation
To justify the **$365/month** base cost of a single AWS server, you would need to process:
`$365 / $0.0006 = ~608,000 resumes / month`

**Conclusion:** Unless processing >500k resumes/month, **OpenAI is significantly cheaper.**

## 3. Recommended Architecture ("Polyglot")
While Production will use OpenAI, the codebase *can* support local development to avoid API fees during testing.

*   **Development (Localhost):**
    *   **Tool:** Ollama.
    *   **Cost:** $0.
    *   **Privacy:** Data never leaves the developer's machine.
*   **Production (Cloud):**
    *   **Tool:** OpenAI API.
    *   **Cost:** Pay-per-use (Negligible).
    *   **Privacy:** OpenAI Zero Data Retention (Enterprise) or Standard API privacy policies (Data not used for training).
