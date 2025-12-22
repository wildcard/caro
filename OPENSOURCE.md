# Open Source at caro

Open source isn't just our distribution model—it's how we think about building software.

## Building in the Open

When we started caro, we made a deliberate choice: develop everything in public from day one. Not because we had to, but because we believe better software emerges when development happens in the open.

Every commit, every design decision, every failed experiment—it's all there in the git history. This isn't always comfortable. Early code is messy. Architectural decisions get revisited. Features get abandoned mid-implementation. But that transparency creates accountability and invites collaboration in ways that polished releases never can.

## Why AGPL-3.0

We chose the GNU Affero General Public License for a specific reason: caro is a tool that generates shell commands using AI. If someone takes this code and runs it as a service, we believe their users deserve access to any improvements they've made.

This isn't about restricting use—commercial use is explicitly allowed. It's about ensuring that the safety improvements, the new patterns we detect, the edge cases we handle, all flow back to the community. When you're building a tool that executes commands on people's systems, the collective wisdom of the community matters.

If AGPL doesn't work for your use case, reach out. We're open to discussing alternatives for genuine needs.

## What's Open

**Everything.**

The entire codebase is open source:
- Core CLI and command generation
- All inference backends (MLX, vLLM, Ollama)
- Safety validation with 52+ dangerous patterns
- The agentic refinement loop
- Platform detection and context awareness
- Our test suite and development tooling

We don't maintain a proprietary version with additional features. We don't hold back security patches for paying customers. The code you see on GitHub is the code we run.

## Safety in the Open

Building an AI tool that generates shell commands requires a particular kind of openness. Our [safety module](src/safety/) isn't just open source—it's designed to be audited, challenged, and improved by anyone who cares about command-line safety.

Every dangerous pattern we detect is visible. Every decision about risk levels is documented. If you find a gap in our safety coverage, that's not a vulnerability to exploit—it's a contribution waiting to happen.

We maintain a [public list of dangerous patterns](src/safety/mod.rs) that we block. We'd rather have attackers know what we catch than have defenders not know what we miss.

## Standing on the Shoulders of Giants

caro exists because of open source:

- **[MLX](https://github.com/ml-explore/mlx)** - Apple's machine learning framework powers our Apple Silicon inference
- **[Candle](https://github.com/huggingface/candle)** - Hugging Face's Rust ML framework enables cross-platform CPU inference
- **[vLLM](https://github.com/vllm-project/vllm)** - High-performance serving for remote inference
- **[Ollama](https://ollama.ai)** - Local LLM runtime that made local inference accessible
- **[clap](https://github.com/clap-rs/clap)** - The CLI framework that handles our argument parsing
- **[tokio](https://tokio.rs)** - Async runtime that powers our backend communication

We upstream fixes when we find them. We document our integration experiences. We try to be the kind of downstream user that maintainers appreciate.

## Community-Driven Development

Open source at caro isn't just about code access—it's about development process:

**Specifications are public.** Our [specs/](specs/) directory contains design documents, architectural decisions, and implementation plans. You can see not just what we built, but why we built it that way.

**Development is visible.** We use GitHub issues for planning, discussions for ideas, and pull requests for all changes. There are no private roadmap meetings. If it affects the project, it happens in public.

**Contributions shape direction.** We don't have a fixed roadmap that ignores community input. Backend implementations, safety patterns, platform support—these can come from anyone.

## Areas for Contribution

We actively welcome contributions in:

- **Safety patterns** - New dangerous command patterns, edge cases, platform-specific risks
- **Backend implementations** - Additional inference backends, optimization improvements
- **Platform support** - Better handling for Linux distributions, BSD variants, Windows edge cases
- **Documentation** - Usage examples, integration guides, troubleshooting
- **Test coverage** - Property-based testing, fuzzing, edge case discovery

See our [Contributing Guidelines](CONTRIBUTING.md) for how to get started.

## The Economics of Open Source

We believe open source can be sustainable without being extractive. caro is currently a community project, but our approach to potential commercialization is straightforward:

- The core tool will always be open source and free
- Any commercial offerings would be services built around the tool, not locked-down versions of it
- Improvements made in commercial contexts flow back to the open source project

We're inspired by projects like [Tailscale](https://tailscale.com/opensource), [Grafana](https://grafana.com/oss/), and [GitLab](https://about.gitlab.com/company/stewardship/) that have shown open source and sustainability aren't mutually exclusive.

## Transparency as a Feature

When you run `caro "delete old log files"`, you're trusting the tool to generate something safe. That trust is easier to extend when:

- You can read exactly how we validate commands
- You can see the model prompts we use
- You can verify our safety patterns match your risk tolerance
- You can fork and modify if our choices don't fit your needs

Closed-source AI tools ask you to trust their judgment. Open-source AI tools let you verify it.

## Get Involved

- **GitHub**: [github.com/wildcard/caro](https://github.com/wildcard/caro)
- **Issues**: Report bugs, request features, ask questions
- **Discussions**: Share ideas, get help, connect with other users
- **Pull Requests**: Contribute code, docs, tests, or translations

The best time to get involved in an open source project is early. We're still small enough that individual contributions meaningfully shape the project's direction.

---

*"The best way to have a good idea is to have lots of ideas."* — Linus Pauling

In open source, the best way to have good ideas is to have lots of people thinking about the problem. That's why we build in the open.
