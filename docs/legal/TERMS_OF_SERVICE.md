# Caro Terms of Service

**Last Updated:** January 2026

## The Essence

Caro is free and open source software licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**. These terms supplement, but do not replace, the rights and obligations under that license.

---

## 1. Acceptance

By using Caro, you agree to:
1. The [AGPL-3.0 License](https://www.gnu.org/licenses/agpl-3.0.html)
2. These Terms of Service
3. Our [Privacy Policy](./PRIVACY_POLICY.md)

If you disagree, don't use the software. Simple as that.

---

## 2. What Caro Is

Caro is a command-line tool that converts natural language to shell commands using local AI inference. It is:

- **A tool, not a service** - You run it; we don't run it for you
- **Open source** - Full source available at [github.com/wildcard/caro](https://github.com/wildcard/caro)
- **Local-first** - Processes everything on your machine
- **Safety-focused** - Validates commands before execution, but **you decide what runs**

---

## 3. Your Responsibilities

### 3.1 Command Review

**You are responsible for reviewing and understanding every command before execution.**

Caro is an AI assistant that generates shell commands. AI can make mistakes. Before running any generated command:

- Read the command carefully
- Understand what it will do
- Consider potential side effects
- Use the safety level appropriate for your environment

### 3.2 System Access

You are responsible for:
- Running Caro with appropriate system privileges
- Understanding the commands you execute
- Any consequences of executing generated commands
- Maintaining backups of important data

### 3.3 Compliance

You agree to use Caro in compliance with:
- All applicable laws and regulations
- Your organization's policies (if applicable)
- System administrator guidelines (if applicable)

---

## 4. What We Provide

### 4.1 The Software

We provide Caro "as is" under AGPL-3.0. This includes:
- Source code on GitHub
- Pre-built binaries via GitHub Releases
- Documentation in the repository

### 4.2 Community Support

Support is community-driven through:
- GitHub Issues for bug reports
- GitHub Discussions for questions
- Pull request reviews for contributions

We do not guarantee response times or resolution of issues.

---

## 5. What We Don't Provide

- **No warranty** - See AGPL-3.0 Section 15
- **No guaranteed uptime** - It's local software on your machine
- **No professional support** - Unless separately arranged
- **No liability for damages** - See AGPL-3.0 Section 16

---

## 6. Disclaimer of Warranty

**THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.**

In plain English:
- We don't guarantee Caro will work perfectly
- We don't guarantee generated commands are correct or safe
- We don't guarantee the software is free of bugs
- **Always review commands before execution**

---

## 7. Limitation of Liability

**IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.**

In plain English:
- If a generated command damages your system, that's on you
- If you lose data, we're not liable
- You use this software at your own risk
- This is why we emphasize reviewing commands before execution

---

## 8. Safety Features

Caro includes safety validation features that:
- Detect potentially dangerous command patterns
- Block known destructive operations in strict mode
- Require confirmation for risky commands
- Warn about privilege escalation

**These features are aids, not guarantees.** They cannot catch every dangerous command. You remain responsible for what you execute.

---

## 9. Intellectual Property

### 9.1 Caro Source Code

Licensed under AGPL-3.0. You may:
- Use, copy, and modify the software
- Distribute the software
- Use it commercially

You must:
- Preserve copyright notices
- Provide source code when distributing
- License modifications under AGPL-3.0
- Disclose source for network-accessible modifications

### 9.2 Kyaro Character

The Kyaro mascot artwork in `assets/kyaro/` is **NOT** covered by AGPL-3.0. It's proprietary. See [assets/kyaro/README.md](../../assets/kyaro/README.md) for terms.

### 9.3 Your Content

Commands you generate and prompts you write remain yours. We don't collect or claim any rights to them.

---

## 10. Contributions

Contributions to Caro are welcomed under our:
- [Contributor License Agreement](./CLA.md)
- [Developer Certificate of Origin](./DCO.txt)
- [Contributing Guidelines](../../CONTRIBUTING.md)

---

## 11. Modifications to Terms

We may update these terms. Changes will be:
- Committed to the repository with a dated changelog
- Noted in release notes when significant

Continued use after changes constitutes acceptance.

---

## 12. Governing Law

These terms are governed by the laws applicable to free software distribution. For disputes:

1. Try GitHub Issues/Discussions first
2. Community mediation is preferred
3. Litigation is a last resort

---

## 13. Severability

If any provision is unenforceable, the remaining provisions stay in effect. We'll replace invalid provisions with enforceable ones that match the original intent.

---

## 14. Entire Agreement

These Terms, the AGPL-3.0 License, and our Privacy Policy constitute the complete agreement regarding Caro usage.

---

## Quick Reference

| Can I... | Answer |
|----------|--------|
| Use Caro for free? | Yes |
| Use Caro commercially? | Yes (AGPL-3.0 terms apply) |
| Modify Caro? | Yes (share modifications under AGPL-3.0) |
| Use Caro offline? | Yes |
| Blame you if I rm -rf /? | No |
| Contribute improvements? | Yes, please! |

---

## Contact

- **Issues**: [github.com/wildcard/caro/issues](https://github.com/wildcard/caro/issues)
- **Discussions**: [github.com/wildcard/caro/discussions](https://github.com/wildcard/caro/discussions)

---

*These terms are provided under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/), inspired by [Automattic's Legalmattic](https://github.com/Automattic/legalmattic).*
