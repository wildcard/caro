# GitHub Project Board Setup - Evaluation Harness Milestone

## Overview

Created a comprehensive GitHub Projects board to track the 16-week evaluation harness maturity roadmap.

**Project URL**: https://github.com/users/wildcard/projects/6
**Milestone URL**: https://github.com/wildcard/caro/milestone/11

---

## Project Configuration

### Basic Details

- **Title**: LLM Evaluation Harness - Maturity Roadmap
- **Project Number**: #6
- **Owner**: wildcard
- **Items**: 8 issues covering 15 work packages
- **Timeline**: 16 weeks (Jan 17 - May 16, 2026)

### Custom Fields

| Field Name | Type | Purpose |
|------------|------|---------|
| **Phase** | Single-select | Tracks which of 8 phases the issue belongs to |
| **Duration** | Text | Timeline for completion (e.g., "Weeks 1-2") |
| **Effort (hours)** | Text | Estimated effort range (e.g., "25-35 hours") |
| **Work Packages** | Text | Work package identifiers (e.g., "WP09-11") |

### Phase Options

- Phase 1: Multi-Backend Validation
- Phase 2: Prompt Engineering Framework
- Phase 3: Model-Specific Intelligence
- Phase 4: Product Feedback Loops
- Phase 5: Fine-Tuning Integration
- Phase 6: Advanced Analytics & Visualization
- Phase 7: Test Coverage & Quality
- Phase 8: Performance & Cost Optimization

---

## Issues with Custom Fields

| Issue | Title | Phase | Duration | Effort | Work Packages |
|-------|-------|-------|----------|--------|---------------|
| [#516](https://github.com/wildcard/caro/issues/516) | Multi-Backend CI Matrix | Phase 1 | Weeks 1-2 | 20-30 hours | WP09 |
| [#517](https://github.com/wildcard/caro/issues/517) | Prompt Engineering Framework | Phase 2 | Weeks 3-4 | 25-35 hours | WP10-11 |
| [#521](https://github.com/wildcard/caro/issues/521) | Model-Specific Intelligence | Phase 3 | Weeks 5-6 | 25-35 hours | WP12-13 |
| [#522](https://github.com/wildcard/caro/issues/522) | Product Feedback Loops | Phase 4 | Weeks 7-8 | 25-35 hours | WP14-15 |
| [#518](https://github.com/wildcard/caro/issues/518) | Fine-Tuning Integration | Phase 5 | Weeks 9-10 | 25-35 hours | WP16-17 |
| [#523](https://github.com/wildcard/caro/issues/523) | Advanced Analytics & Visualization | Phase 6 | Weeks 11-12 | 30-40 hours | WP18-19 |
| [#524](https://github.com/wildcard/caro/issues/524) | Test Coverage & Quality | Phase 7 | Weeks 13-14 | 30-40 hours | WP20-21 |
| [#525](https://github.com/wildcard/caro/issues/525) | Performance & Cost Optimization | Phase 8 | Weeks 15-16 | 25-35 hours | WP22-23 |

**Total Effort**: 205-275 hours (≈320 hours budgeted with buffer)

---

## Available Views

The project board supports multiple visualization modes:

### 1. Table View (Default)
- Spreadsheet-style layout
- All custom fields visible in columns
- Sortable and filterable
- Best for detailed tracking

### 2. Board View
- Kanban-style cards
- Group by Status: Todo → In Progress → Done
- Drag-and-drop workflow
- Best for workflow management

### 3. Roadmap View
- Timeline visualization
- Duration-based Gantt chart
- Phase sequencing visible
- Best for stakeholder communication

---

## Workflow Integration

### Status Tracking

Default status field values:
- **Todo**: Issue not yet started
- **In Progress**: Currently being worked on
- **Done**: Completed and verified

### Milestone Integration

All issues are linked to:
- **Milestone #11**: LLM Evaluation Harness - Maturity & Quality Confidence
- **Due Date**: May 16, 2026

### Automation Opportunities

Consider adding project automations:
- Auto-move to "In Progress" when issue is assigned
- Auto-move to "Done" when issue is closed
- Auto-add new issues with milestone #11

---

## How to Use the Board

### For Developers

1. **Pick Next Task**: Filter by Phase, choose next sequential work package
2. **Track Progress**: Move cards through Todo → In Progress → Done
3. **Update Estimates**: Refine Effort field as work progresses
4. **Link PRs**: GitHub automatically shows linked pull requests

### For Project Managers

1. **View Progress**: Use Roadmap view to see timeline
2. **Check Status**: Table view shows all fields at once
3. **Report**: Export to CSV for stakeholder updates
4. **Forecast**: Use Effort totals to estimate completion

### For Stakeholders

1. **High-Level View**: Roadmap view shows overall timeline
2. **Phase Completion**: Group by Phase to see progress
3. **Effort Tracking**: Sum Effort field to see work invested
4. **Deliverables**: Each issue links to specific deliverables

---

## Technical Implementation

### GraphQL API Usage

Custom fields were populated using GitHub's GraphQL API v2:

```bash
# Field IDs
PROJECT_ID="PVT_kwHOACzaDc4BM0Fu"
PHASE_FIELD_ID="PVTSSF_lAHOACzaDc4BM0Fuzg7_Xmw"
DURATION_FIELD_ID="PVTF_lAHOACzaDc4BM0Fuzg7_Xnc"
EFFORT_FIELD_ID="PVTF_lAHOACzaDc4BM0Fuzg7_XoI"
WP_FIELD_ID="PVTF_lAHOACzaDc4BM0Fuzg7_XoM"

# Update mutation example
gh api graphql -f query='
mutation {
  updateProjectV2ItemFieldValue(input: {
    projectId: "'"$PROJECT_ID"'"
    itemId: "PVTI_..."
    fieldId: "'"$PHASE_FIELD_ID"'"
    value: {singleSelectOptionId: "a22a296c"}
  }) {
    projectV2Item { id }
  }
}'
```

---

## Next Steps

### Immediate
1. ✅ Project created
2. ✅ Issues added
3. ✅ Custom fields populated
4. ✅ Documentation complete

### Ongoing
1. Update Status field as work progresses
2. Link PRs to issues
3. Refine Effort estimates based on actuals
4. Add notes/comments to capture decisions

### Future Enhancements
1. Add automation rules
2. Create custom views for specific audiences
3. Export reports for monthly reviews
4. Archive completed phases

---

## Related Documentation

- **Milestone Plan**: `thoughts/shared/plans/evaluation-harness-maturity-milestone.md`
- **Executive Summary**: `thoughts/shared/plans/evaluation-harness-milestone-summary.md`
- **Issue #135**: Parent tracking issue for evaluation harness

---

**Created**: 2026-01-17
**Last Updated**: 2026-01-17
**Status**: Active
