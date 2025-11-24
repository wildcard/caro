# cmdai Launch Infrastructure

> **Complete GitHub setup for V1.0 launch and beyond**

This directory contains all the GitHub infrastructure needed to organize cmdai development from MVP through Series A.

---

## What's In This Directory

### 1. **GITHUB_ISSUES.md**
15 ready-to-create issues for V1.0 launch, including:
- Performance optimization
- Binary size reduction
- Package distribution (Homebrew, crates.io, apt)
- Testing, documentation, security
- Release automation

**Priority breakdown**:
- P0 (Critical): 7 issues
- P1 (High): 5 issues
- P2 (Medium): 3 issues

**Total effort**: 38 story points (~6-8 weeks with 2-3 contributors)

### 2. **GITHUB_LABELS.json**
29 labels organized by:
- **Type** (6): feature, bug, docs, refactor, test, infra
- **Priority** (4): P0 (critical), P1 (high), P2 (medium), P3 (low)
- **Component** (7): cli, backend, safety, cloud, enterprise, workflows, integrations
- **Phase** (6): mvp, v1.0, q1-2025, q2-2025, q3-2025, q4-2025
- **Other** (6): good-first-issue, help-wanted, blocked, needs-design, needs-review, breaking-change

**Format**: JSON array ready for bulk import via GitHub CLI or API

### 3. **GITHUB_MILESTONES.md**
5 detailed milestone descriptions:

| Milestone | Timeline | Revenue Target | Key Focus |
|-----------|----------|----------------|-----------|
| v1.0 - Production CLI | Dec 2025 | $0 (OSS) | Performance, distribution, docs |
| Q1 2025 - Cloud Launch | Jan-Mar 2025 | $2K MRR | Cloud API, team features |
| Q2 2025 - Enterprise | Apr-Jun 2025 | $150K ARR | Audit logs, RBAC, SSO |
| Q3 2025 - Platform | Jul-Sep 2025 | $500K ARR | Workflows, integrations, marketplace |
| Q4 2025 - Scale | Oct-Dec 2025 | $1.2M ARR | Series A, team of 8 |

### 4. **IMPORT_GUIDE.md**
Step-by-step instructions for:
- Importing labels (3 methods: CLI, web UI, script)
- Creating milestones
- Creating issues
- Setting up GitHub Projects
- Configuring repository settings

**Time required**: ~30 minutes

---

## Quick Start

### Option 1: Automated Import (Fastest)

```bash
cd /home/user/cmdai/launch

# 1. Import labels
cat GITHUB_LABELS.json | jq -r '.[] | @json' | while read label; do
  name=$(echo $label | jq -r '.name')
  color=$(echo $label | jq -r '.color')
  description=$(echo $label | jq -r '.description')
  gh label create "$name" --color "$color" --description "$description" --repo wildcard/cmdai || true
done

# 2. Create milestones manually (see GITHUB_MILESTONES.md)
#    Go to: https://github.com/wildcard/cmdai/milestones

# 3. Create issues manually (see GITHUB_ISSUES.md)
#    Go to: https://github.com/wildcard/cmdai/issues/new
```

### Option 2: Manual Import via Web UI

1. **Labels**: https://github.com/wildcard/cmdai/labels
   - Click "New label" 29 times
   - Copy from `GITHUB_LABELS.json`

2. **Milestones**: https://github.com/wildcard/cmdai/milestones
   - Click "New milestone" 5 times
   - Copy from `GITHUB_MILESTONES.md`

3. **Issues**: https://github.com/wildcard/cmdai/issues
   - Create 15 issues from `GITHUB_ISSUES.md`
   - Start with P0 issues

**See `IMPORT_GUIDE.md` for detailed instructions**

---

## File Structure

```
launch/
├── README.md                  # This file
├── GITHUB_ISSUES.md          # 15 V1.0 issues (ready to copy-paste)
├── GITHUB_LABELS.json        # 29 labels (importable JSON)
├── GITHUB_MILESTONES.md      # 5 milestone descriptions
└── IMPORT_GUIDE.md           # Step-by-step import instructions
```

---

## What's Already in `.github/`

The following are already committed to the repository:

### Issue Templates
- `bug_report.yml` - Bug reporting form
- `feature_request.yml` - Feature request form
- `backend_integration.yml` - Backend integration requests
- `safety_pattern.yml` - Safety pattern submissions
- `good_first_issue.yml` - **NEW**: Template for creating beginner-friendly issues

### Pull Request Template
- `PULL_REQUEST_TEMPLATE.md` - PR checklist and guidelines

### Workflows (CI/CD)
- GitHub Actions workflows in `.github/workflows/`

---

## Usage Workflow

### For Maintainers

**Setting up (one-time)**:
1. Import labels → `IMPORT_GUIDE.md`
2. Create milestones → `GITHUB_MILESTONES.md`
3. Create initial issues → `GITHUB_ISSUES.md`
4. Set up project board → `IMPORT_GUIDE.md` Step 5

**Ongoing**:
1. Triage new issues within 48 hours
2. Label issues appropriately
3. Assign to milestones
4. Keep project board updated
5. Close completed issues

### For Contributors

**Finding work**:
1. Browse issues: https://github.com/wildcard/cmdai/issues
2. Filter by labels:
   - `good-first-issue` - Beginner-friendly
   - `help-wanted` - Needs contributors
   - `phase: v1.0` - Current priority
3. Comment "I'd like to work on this"
4. Wait for assignment
5. Start coding!

**Tracking progress**:
- Check project board: https://github.com/wildcard/cmdai/projects
- View milestone progress: https://github.com/wildcard/cmdai/milestones

---

## Issue Labels Quick Reference

### Most Important Labels

**Priority** (use one):
- `priority: P0` 🔴 Critical, blocking launch
- `priority: P1` 🟡 High priority, needed for V1.0
- `priority: P2` 🟢 Medium priority, nice to have
- `priority: P3` ⚪ Low priority, future

**Phase** (use one):
- `phase: v1.0` - Production-ready CLI
- `phase: q1-2025` - Cloud features
- `phase: q2-2025` - Enterprise features
- `phase: q3-2025` - Platform features
- `phase: q4-2025` - Scale and fundraise

**Type** (use one):
- `type: feature` - New functionality
- `type: bug` - Something broken
- `type: docs` - Documentation
- `type: test` - Testing improvements
- `type: infra` - CI/CD, releases, deployment

**Component** (use one or more):
- `component: cli` - CLI interface
- `component: backend` - LLM backends
- `component: safety` - Safety validation
- `component: cloud` - Cloud/SaaS features

**Special**:
- `good-first-issue` - For new contributors
- `help-wanted` - Need community help
- `blocked` - Can't proceed yet

---

## Milestone Timeline

```
NOW                                                          END OF 2025
 |                                                                |
 v                                                                v
[MVP] → [v1.0] → [Q1 2025] → [Q2 2025] → [Q3 2025] → [Q4 2025]
         Dec      Jan-Mar     Apr-Jun     Jul-Sep     Oct-Dec

   ↓        ↓          ↓           ↓           ↓           ↓
 MVP     Launch    Cloud     Enterprise  Platform   Series A
 Work     CLI       API        RBAC       Workflows  Fundraise
         <50MB    JWT/Auth   Audit logs  DAG engine   $5-10M
         Docs     Teams      SSO/SAML   Marketplace  8 people
```

---

## Key Metrics by Milestone

| Milestone | GitHub Stars | Downloads | MRR | ARR |
|-----------|-------------|-----------|-----|-----|
| v1.0 (Dec 2025) | 5,000 | 10,000 | $0 | $0 |
| Q1 2025 | 10,000 | 50,000 | $2K | $24K |
| Q2 2025 | 15,000 | 100,000 | $15K | $180K |
| Q3 2025 | 20,000 | 200,000 | $50K | $600K |
| Q4 2025 | 25,000 | 500,000 | $100K | $1.2M |

---

## Next Steps

### Immediately (This Week)
- [ ] Import all labels to GitHub
- [ ] Create all 5 milestones
- [ ] Create the 7 P0 issues for V1.0
- [ ] Set up project board
- [ ] Enable GitHub Discussions

### This Month
- [ ] Create all 15 V1.0 issues
- [ ] Assign issues to contributors
- [ ] Start working on P0 issues
- [ ] Weekly milestone progress reviews

### Before Launch (6-8 weeks)
- [ ] Complete all P0 issues
- [ ] Complete most P1 issues
- [ ] All tests passing
- [ ] Documentation polished
- [ ] Release V1.0

---

## Resources

### Internal Documentation
- [MVP_TO_V1.md](../MVP_TO_V1.md) - V1.0 completion guide
- [ROADMAP.md](../ROADMAP.md) - Full product roadmap
- [GITHUB_SETUP.md](../GITHUB_SETUP.md) - Project management guide
- [BUSINESS_MODEL.md](../BUSINESS_MODEL.md) - Business strategy

### GitHub Links
- **Issues**: https://github.com/wildcard/cmdai/issues
- **Milestones**: https://github.com/wildcard/cmdai/milestones
- **Projects**: https://github.com/wildcard/cmdai/projects
- **Discussions**: https://github.com/wildcard/cmdai/discussions
- **Labels**: https://github.com/wildcard/cmdai/labels

### External Resources
- [GitHub CLI Documentation](https://cli.github.com/manual/)
- [GitHub Projects Documentation](https://docs.github.com/en/issues/planning-and-tracking-with-projects)
- [Issue Template Documentation](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests)

---

## Troubleshooting

### Labels not importing?
- Check GitHub CLI authentication: `gh auth status`
- Verify repository access: `gh repo view wildcard/cmdai`
- Use web UI as fallback

### Can't create milestones via CLI?
- Use web UI instead (faster for one-time setup)
- See `IMPORT_GUIDE.md` for manual instructions

### Issue templates not showing?
- Templates must be in `.github/ISSUE_TEMPLATE/`
- Filenames must end in `.yml` or `.yaml`
- Check YAML syntax with `yamllint`

---

## Contributing to This Infrastructure

If you improve these GitHub resources:

1. Update files in `launch/` directory
2. Update this README if adding new resources
3. Submit PR with changes
4. Tag maintainers for review

---

## Questions?

- **GitHub setup issues**: Create a discussion in [Q&A](https://github.com/wildcard/cmdai/discussions/categories/q-a)
- **Project management**: See [GITHUB_SETUP.md](../GITHUB_SETUP.md)
- **Roadmap questions**: See [ROADMAP.md](../ROADMAP.md)

---

**Everything you need to organize cmdai from MVP to Series A. Let's build!** 🚀
