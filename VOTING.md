# Community Feature Voting Guide

## Overview

cmdai is a community-driven project. **You** decide which features get built next! This document explains how the democratic feature voting process works.

## Voting Schedule

### Quarterly Planning Cycles

cmdai follows a **quarterly planning cycle**:

```
Q1 (Jan-Mar): Planning → Voting → Implementation → Release
Q2 (Apr-Jun): Planning → Voting → Implementation → Release
Q3 (Jul-Sep): Planning → Voting → Implementation → Release
Q4 (Oct-Dec): Planning → Voting → Implementation → Release
```

**Key Dates** (approximate):
- **Week 1**: Voting opens for all feature proposals
- **Week 2**: Community discussion and refinement
- **Week 3**: Final voting, top features selected
- **Week 4-11**: Implementation of selected features
- **Week 12**: Release and retrospective

## How to Vote

### 1. Browse Feature Proposals

Visit [GitHub Discussions → Feature Requests](https://github.com/wildcard/cmdai/discussions/categories/feature-requests)

Features are organized by **Theme**:
- 🤖 Theme 1: Advanced Model Support
- ⚙️ Theme 2: Command Execution & Workflows
- 🔌 Theme 3: Integration & Ecosystem
- 🧠 Theme 4: AI/LLM Enhancements
- 🌍 Theme 5: Platform & Language Support
- 🔒 Theme 6: Enterprise & Security
- 🛠️ Theme 7: Developer Experience
- 📚 Theme 8: Education & Onboarding

### 2. Use Reactions to Vote

**Voting is simple - just use emoji reactions:**

| Reaction | Meaning | Weight |
|----------|---------|--------|
| 👍 | I want this feature | +3 points |
| ❤️ | I REALLY want this | +5 points |
| 👀 | I'm interested | +1 point |
| 👎 | I don't want this | -2 points |
| 🚀 | High priority | +4 points |

**Voting Power**:
- All users: 1 vote per reaction type
- Contributors (1+ merged PR): 2x multiplier
- Maintainers: Advisory, does not override community

### 3. Comment with Use Cases

**Comments are just as important as votes!**

Help build a strong case for a feature:
- Share your specific use case
- Explain how it would help you
- Suggest implementation ideas
- Raise concerns or edge cases

**Example**:
```markdown
👍 I would use this daily in my DevOps workflow!

Use case: When deploying to production, I need to generate complex
kubectl commands. Currently I have to look up syntax every time.

With this feature, I could just describe what I want to deploy and
cmdai would generate the correct kubectl command with all the flags.
```

### 4. Propose New Features

Don't see your idea? **Create a new feature proposal!**

1. Go to [Discussions](https://github.com/wildcard/cmdai/discussions)
2. Click "New Discussion"
3. Select "Feature Request" template
4. Fill out the template completely
5. Submit and share with the community

**Tips for good proposals**:
- ✅ Clear problem statement
- ✅ Concrete use cases
- ✅ UI mockup or example
- ✅ Consider alternatives
- ❌ Vague ideas without use cases
- ❌ Features that already exist
- ❌ Duplicate proposals (search first!)

## Feature Selection Process

### Scoring Algorithm

Features are ranked by a **Priority Score**:

```
Priority Score = (Vote Points + Impact Score) / (Effort Score + Complexity)
```

**Vote Points**: Sum of all emoji reactions with weights

**Impact Score** (assessed by maintainers):
- 10: Benefits all users, unlocks major workflows
- 7: Benefits most users, significant improvement
- 5: Benefits specific user segment, useful
- 3: Niche use case, minor improvement
- 1: Very narrow use case

**Effort Score** (estimated by maintainers):
- 1: Small (1 week or less)
- 2: Medium (2-3 weeks)
- 4: Large (1-2 months)
- 8: Extra Large (2+ months)

**Complexity** (technical difficulty):
- 1: Simple, low risk
- 2: Moderate, some risk
- 4: Complex, significant risk
- 8: Very complex, high risk

### Selection Criteria

Each quarter, we select **3-5 features** for implementation:

1. **Top Priority Score**: Highest scores get priority
2. **Theme Balance**: Mix of different themes (not all from one theme)
3. **Feasibility**: Can be completed in the quarter
4. **Dependencies**: Features that unlock other features
5. **Maintenance**: Does it create technical debt?

**Example Selection**:
```
Q2 2026 Selected Features:
1. Command Explanation (Score: 4.2, Theme 4)
2. Shell Integration (Score: 3.8, Theme 3)
3. Learning Mode (Score: 3.5, Theme 8)
4. Safe Command Execution (Score: 3.2, Theme 2)
```

### Transparency

All scoring and decisions are **public**:
- Vote counts published in discussion thread
- Scoring spreadsheet shared
- Selection rationale explained
- Community can challenge decisions

## Feature Implementation

### Spec-Driven Development

All selected features follow our **spec-driven workflow**:

1. **Specification** (`/specify`): Create detailed spec
2. **Community Review**: Public review period (1 week)
3. **Planning** (`/plan`): Technical design document
4. **Task Breakdown** (`/tasks`): Ordered implementation tasks
5. **Implementation** (`/implement`): Build the feature
6. **Testing & QA**: Comprehensive testing
7. **Documentation**: User docs and examples
8. **Release**: Include in next version

### Who Implements?

Features can be implemented by:
- **Maintainers**: Core team members
- **Contributors**: Community volunteers
- **Feature Authors**: You proposed it, you can build it!

**Want to implement a feature?**
1. Comment "I'd like to implement this"
2. Maintainers will work with you on the spec
3. Follow the spec-driven development process
4. Submit PR when ready

## Voting Best Practices

### For Voters

**DO**:
- ✅ Vote on multiple features you care about
- ✅ Explain your use case in comments
- ✅ Update your vote if you change your mind
- ✅ Engage constructively in discussions
- ✅ Consider effort vs. impact trade-offs

**DON'T**:
- ❌ Spam or brigade voting
- ❌ Downvote without explanation
- ❌ Demand features without use cases
- ❌ Attack proposers or voters
- ❌ Vote manipulate with fake accounts

### For Feature Proposers

**DO**:
- ✅ Research existing features first
- ✅ Provide detailed use cases
- ✅ Be open to feedback
- ✅ Refine based on comments
- ✅ Offer to help implement

**DON'T**:
- ❌ Propose vague ideas
- ❌ Ignore critical feedback
- ❌ Duplicate existing proposals
- ❌ Demand immediate implementation
- ❌ Take rejection personally

## Voting FAQ

### Can I vote for features in multiple themes?
**Yes!** Vote for as many features as you want across all themes.

### What if my feature doesn't get selected?
Features remain open for future voting rounds. If a feature consistently gets votes, it will eventually be selected. You can also implement it yourself!

### Can I change my vote?
**Yes!** Just change your reaction emoji. We use the current state at voting deadline.

### What if two features conflict?
The community will discuss and possibly merge proposals or vote on alternatives.

### Can maintainers override votes?
Maintainers can veto features for **technical or safety reasons** (e.g., impossible to implement, security risk). This is rare and always explained publicly.

### How are experimental features handled?
Some features may be implemented as **experimental** (behind feature flag) to gather feedback before full commitment.

### What about urgent bug fixes?
Bug fixes and security patches are **not subject to voting** - they are handled immediately.

### Can I propose a feature outside the quarterly cycle?
**Yes!** Feature proposals are accepted year-round. They enter the queue for the next voting cycle.

## Voting History

### Q4 2025 (Hypothetical Example)

**Features Proposed**: 42
**Total Votes Cast**: 1,247
**Selected Features**: 4

| Feature | Theme | Votes | Score | Status |
|---------|-------|-------|-------|--------|
| Command Explanation | 4 | 187 | 4.2 | ✅ Implemented in v1.1 |
| Shell Integration | 3 | 156 | 3.8 | ✅ Implemented in v1.1 |
| Learning Mode | 8 | 134 | 3.5 | ✅ Implemented in v1.1 |
| Safe Execution | 2 | 112 | 3.2 | 🚧 In Progress |

**Notable Features Not Selected**:
- Multi-Model Ensemble (Score: 1.2) - High effort, low priority
- API Server Mode (Score: 2.1) - Deferred to Q1 2026
- Custom Model Support (Score: 2.8) - Dependencies not ready

## Get Started

Ready to shape the future of cmdai?

1. **Browse**: [Current Feature Proposals](https://github.com/wildcard/cmdai/discussions/categories/feature-requests)
2. **Vote**: Add reactions to features you care about
3. **Discuss**: Share your use cases and ideas
4. **Propose**: Create new feature proposals
5. **Build**: Implement features you're passionate about

**Every vote matters. Your voice shapes cmdai!** 🗳️

---

## Questions?

- 💬 Ask in [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)
- 📖 Read the [ROADMAP.md](./ROADMAP.md)
- 🤝 See [CONTRIBUTING.md](./CONTRIBUTING.md)

Thank you for being part of the cmdai community! 🚀
