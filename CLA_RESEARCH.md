# CLA Research for AGPL-3.0 Projects

## Executive Summary

This document summarizes research on Contributor License Agreements (CLAs) for AGPL-3.0 open source projects and provides recommendations for cmdai.

**Date**: December 9, 2025
**Research Focus**: CLA best practices for AGPL-3.0 licensed projects

---

## Research Findings

### 1. CLAs in AGPL Projects

AGPL projects use CLAs for various reasons:

#### **Dual Licensing Use Cases**

Many AGPL projects with CLAs use them to enable dual licensing:

- **Element/Matrix**: Uses AGPL-3.0 + CLA to sell proprietary licenses to enterprises
- **Elasticsearch**: Combines AGPL with ELv2 and SSPL licensing
- **ONLYOFFICE**: Maintains both AGPL and proprietary licensing options
- **Wolkenkit**: Offers AGPL-3.0 alongside commercial licenses

#### **Pure Open Source Use Cases**

Some AGPL projects use CLAs without dual licensing plans:

- **Joplin**: Switched to AGPL-3.0 with CLA to ensure copyright remains with the organization
- **Atuin Desktop**: Uses a simple CLA based on Apache Foundation's template

### 2. CLA vs DCO Debate

#### **Contributor License Agreement (CLA)**

**Advantages**:
- Clear legal documentation of contributor rights
- Allows potential relicensing if needed
- Provides patent grant protections
- Used by major projects (Apache, Google, Microsoft)

**Disadvantages**:
- Seen as "paperwork" barrier to contribution
- Can be perceived as unfriendly to contributors
- Gives maintainers more rights than regular contributors
- May reduce contributor participation

**Controversy**: High-profile cases like MongoDB (2019) and Elasticsearch (2021) used CLA rights to switch to non-open-source licenses, damaging community trust.

#### **Developer Certificate of Origin (DCO)**

**Advantages**:
- Lightweight, low-barrier approach
- Used by Linux kernel, Node.js, Spring Framework
- Simple git sign-off: `git commit -s`
- No extra rights granted to maintainers
- Recently gaining popularity (Spring just adopted it in 2025)

**Disadvantages**:
- No explicit patent grant
- Cannot enable future relicensing
- Less comprehensive legal protection

### 3. Community Trends

**Recent Movement Toward DCO**:
- Spring Framework eliminated CLA in favor of DCO (January 2025)
- Node.js switched from CLA to DCO, seeing increased contributions
- Keycloak adopted DCO in 2023
- Many new projects prefer DCO to reduce friction

**CLAs Still Common For**:
- Projects with commercial backing
- Dual-licensing strategies
- Projects wanting maximum legal flexibility
- Large corporate-sponsored projects

---

## Recommendations for cmdai

### Current Project Context

cmdai is:
- Licensed under AGPL-3.0 only (no dual licensing)
- Community-driven open source project
- Focused on welcoming contributors
- Not planning proprietary licensing

### Option 1: CLA (Implemented)

**File**: [CLA.md](CLA.md)

**Based on**: Apache Software Foundation's Individual CLA v2.2, adapted for AGPL-3.0

**Key Features**:
- Explicit AGPL-3.0 commitment (no dual licensing without community consent)
- Patent grant protections
- Clear contributor representations
- Simple GitHub comment signature: "I have read the CLA Document and I hereby sign the CLA"
- Network copyleft awareness (AGPL Section 13)

**When to Use**: If you want comprehensive legal protection and don't mind a small contribution barrier.

### Option 2: DCO (Also Implemented)

**File**: [DCO.txt](DCO.txt)

**Based on**: Linux Foundation's DCO 1.1 (used by Linux kernel)

**Key Features**:
- Lightweight sign-off via git: `git commit -s`
- No extra rights granted to maintainers
- Simple certification of contribution rights
- Low barrier to entry

**When to Use**: If you want to maximize community participation and are comfortable with less comprehensive legal documentation.

### Option 3: Dual Approach (Recommended)

**Implementation**: Allow contributors to choose either CLA or DCO

**Advantages**:
- Flexibility for different contributor preferences
- Maximizes contribution accessibility
- Provides legal protection for those who want it
- Aligns with modern open source trends

**Documentation**: Updated [CONTRIBUTING.md](CONTRIBUTING.md) to explain both options

---

## Implementation Status

### ✅ Completed

1. **CLA.md** - Full Contributor License Agreement adapted for AGPL-3.0
2. **DCO.txt** - Developer Certificate of Origin (DCO 1.1)
3. **CONTRIBUTING.md** - Updated with CLA/DCO section including:
   - Why cmdai uses a CLA
   - How to sign the CLA
   - DCO alternative instructions
   - FAQ addressing common concerns
   - Commitment to AGPL-3.0 (no proprietary licensing)

### 📋 Recommended Next Steps

1. **Choose Your Approach**:
   - **CLA only**: Remove DCO references from CONTRIBUTING.md
   - **DCO only**: Remove CLA.md and update CONTRIBUTING.md
   - **Both (recommended)**: Keep current implementation

2. **Automate CLA/DCO Checking** (optional but recommended):
   - GitHub Action: [CLA Assistant](https://github.com/contributor-assistant/github-action)
   - DCO bot: [Probot DCO](https://github.com/probot/dco)
   - Manual tracking: Spreadsheet or GitHub issue

3. **Update Pull Request Template** (optional):
   - Add CLA/DCO reminder to `.github/PULL_REQUEST_TEMPLATE.md`
   - Include checkbox: "[ ] I have signed the CLA or used `git commit -s`"

4. **Announce to Community**:
   - Create GitHub Discussion explaining CLA/DCO implementation
   - Update project README with link to CONTRIBUTING.md
   - Consider blog post explaining reasoning

---

## References

### Articles and Guides

- [CLA vs. DCO: What's the difference? | Opensource.com](https://opensource.com/article/18/3/cla-vs-dco-whats-difference)
- [The Developer Certificate of Origin is a great alternative to a CLA](https://drewdevault.com/2021/04/12/DCO.html)
- [Hello DCO, Goodbye CLA: Simplifying Contributions to Spring](https://spring.io/blog/2025/01/06/hello-dco-goodbye-cla-simplifying-contributions-to-spring/)
- [The Legal Side of Open Source | Open Source Guides](https://opensource.guide/legal/)
- [Contributor License Agreements | Google Open Source](https://opensource.google/documentation/reference/cla)

### AGPL-Specific Resources

- [Joplin is switching to the GNU Affero General Public License v3 (AGPL)](https://joplinapp.org/news/20221221-agpl/)
- [Sustainable licensing at Element with AGPL](https://element.io/blog/sustainable-licensing-at-element-with-agpl/)
- [AGPL License Overview | OpenZeppelin](https://www.openzeppelin.com/agpl-license)
- [Add FAQ item on why "AGPL + CLA" is a poor "fair source" license](https://github.com/fairsource/fair.io/issues/58)
- [Are AGPL and CLA compatible? | ONLYOFFICE](https://github.com/ONLYOFFICE/DocumentServer/issues/2288)

### Example CLAs

- [Atuin Desktop CLA](https://github.com/atuinsh/desktop/blob/main/CLA.md)
- [Apache Software Foundation CLA](https://www.apache.org/licenses/icla.pdf)
- [Project Harmony CLA Templates](http://www.harmonyagreements.org/)

### DCO Resources

- [Developer Certificate of Origin - Wikipedia](https://en.wikipedia.org/wiki/Developer_Certificate_of_Origin)
- [Developer Certificate of Origin - Keycloak](https://www.keycloak.org/2023/10/dco)
- [CLAs And DCOs | FINOS](https://osr.finos.org/docs/bok/artifacts/clas-and-dcos/)

---

## Conclusion

For cmdai as a pure AGPL-3.0 project without dual licensing plans, I recommend:

1. **Primary recommendation**: Support both CLA and DCO (current implementation)
2. **Emphasize**: No dual licensing - all contributions remain AGPL-3.0
3. **Consider**: Automating CLA/DCO verification with GitHub Actions
4. **Monitor**: Community feedback and adjust if CLA creates contribution barriers

The dual approach provides legal clarity while maintaining a welcoming, low-barrier contribution process aligned with modern open source best practices.
