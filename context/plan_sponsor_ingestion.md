# Plan: Accommodate Sponsor Project Format

## Goal
Update the `ProjectService` to support the specific Excel/CSV headers provided by the project sponsor while preserving existing functionality and avoiding immediate database migrations.

## Sponsor Format Analysis
The sponsor's format includes several fields not directly matching our internal schema:
- **Title**: Maps to `title`.
- **Start / Duration**: Not in schema. Will be concatenated into `description`.
- **Intern Count**: Maps to `intern_cap`.
- **Term**: Maps to `term`.
- **Description**: Maps to `description`.
- **Deliverable**: Maps to `deliverable`.
- **Manager**: Maps to `manager`.
- **Experience**: Maps to `requirements`.
- **Priority Level**: Maps to `priority` (requires string-to-int conversion).

## Implementation Steps

### 1. Update `ProjectData` Struct (`src/service.rs`)
- Add fields for `deliverable`, `start`, and `duration`.
- Change `priority` from `i16` to `String` to handle textual labels like "High".
- Add `serde` aliases for sponsor-specific headers:
    - `intern_cap`: alias "Intern Count"
    - `requirements`: alias "Experience"
    - `priority`: alias "Priority Level"

### 2. Update `parse_excel` Logic (`src/service.rs`)
- Expand the manual mapping loop to include the new headers:
    - "experience" -> `requirements`
    - "intern count" -> `intern_cap`
    - "priority level" -> `priority`
    - "deliverable", "start", "duration" -> new struct fields.

### 3. Refine `insert_projects` Logic (`src/service.rs`)
- **Priority Mapper**: Implement a helper to map `priority` string values:
    - "High" -> 3
    - "Moderate" / "moderate" -> 2
    - "Low" -> 1
    - Default -> 0
- **String Concatenation**: Prepend `[Start: {start}, Duration: {duration}]` to the `description` if those values are present.
- **SQL Update**: Update the `INSERT` query to include the `deliverable` column.

## Verification Plan
- Run the orchestrator locally.
- Use `curl` or a REST client to trigger an ingestion using `test-project-sheets/Intern Project Sample.csv`.
- Verify the `projects` table in Supabase reflects the correct mapping and concatenated descriptions.
