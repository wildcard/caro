---
theme: seriph
background: https://images.unsplash.com/photo-1518770660439-4636190af475?q=80&w=2574
title: 'Caro: Your AI Shell Companion'
info: |
  ## Caro Investor Pitch Deck

  Production-ready AI shell assistant for air-gapped enterprise environments

  Offline-first • Safety-first • Enterprise-ready
class: text-center
drawings:
  persist: false
transition: slide-left
mdc: true
highlighter: shiki
colorSchema: dark
---

# Caro

### Your AI Shell Companion

<div class="pt-12">
  <span @click="$slidev.nav.next" class="px-6 py-3 rounded-lg bg-blue-600 bg-opacity-80 backdrop-blur cursor-pointer hover:bg-opacity-100 transition">
    Investor Pitch Deck →
  </span>
</div>

<div class="absolute bottom-10 text-sm opacity-50">
  Production-ready • Published on crates.io • v0.1.0
</div>

---
layout: center
class: text-center
---

# Executive Summary

<div class="text-left max-w-4xl mx-auto mt-8">

**Caro** is a production-ready AI shell assistant that works **locally**, even in air-gapped environments.

<v-clicks>

- 🔒 **Offline-first**: Embedded models, no internet required
- ⚡ **GPU accelerated**: <2s inference on Apple Silicon
- 🛡️ **Safety-first**: 52 pre-compiled safety patterns
- 🏢 **Enterprise-ready**: Published on crates.io v0.1.0
- 🌐 **Live now**: caro.sh

</v-clicks>

</div>

<div v-click class="absolute bottom-10 left-0 right-0 text-center">
  <div class="text-2xl font-bold text-blue-400">
    Serving the 8M developers who can't use cloud AI tools
  </div>
</div>

---
layout: two-cols
---

# The Problem

<div class="pr-4">

## Terminal Users Face Three Critical Challenges

<v-clicks>

### 1. Command Complexity 😵
Even experienced devs struggle with syntax
- "man PS" documentation overload
- 200+ flags for common tools

### 2. Context Switching 🔄
Terminal ↔ Browser disrupts flow
- Costs 15-20 minutes daily
- Breaking concentration repeatedly

### 3. Enterprise Restrictions 🔐
Organizations limit AI tool usage
- Financial services, healthcare, government
- Air-gapped environments

</v-clicks>

</div>

::right::

<div class="pl-4 mt-16">

<v-click>

## Market Pain Point

<div class="p-6 bg-red-900 bg-opacity-30 rounded-lg mt-4">
  <div class="text-4xl font-bold text-red-400 mb-2">73%</div>
  <div class="text-lg">
    of enterprise developers <strong>cannot use</strong> cloud-based AI coding assistants due to security policies
  </div>
</div>

</v-click>

<v-click>

<div class="mt-8 p-4 bg-blue-900 bg-opacity-30 rounded-lg">
  <div class="text-2xl font-bold text-blue-400 mb-2">8M Developers</div>
  <div>
    locked out of the AI productivity revolution
  </div>
</div>

</v-click>

</div>

---
layout: center
---

# Our Solution: Caro

<div class="grid grid-cols-2 gap-6 mt-8 max-w-5xl mx-auto">

<v-clicks>

<div class="p-6 bg-blue-900 bg-opacity-20 rounded-lg">
  <div class="text-4xl mb-2">🔌</div>
  <div class="text-xl font-bold mb-2">Works Offline</div>
  <div class="text-sm opacity-80">
    Embedded Qwen2.5-Coder-1.5B-Instruct model<br/>
    No internet required after setup
  </div>
</div>

<div class="p-6 bg-green-900 bg-opacity-20 rounded-lg">
  <div class="text-4xl mb-2">⚡</div>
  <div class="text-xl font-bold mb-2">GPU Accelerated</div>
  <div class="text-sm opacity-80">
    Metal GPU on Apple Silicon<br/>
    <2s inference time (M1/M2/M3/M4)
  </div>
</div>

<div class="p-6 bg-purple-900 bg-opacity-20 rounded-lg">
  <div class="text-4xl mb-2">📦</div>
  <div class="text-xl font-bold mb-2">Simple Installation</div>
  <div class="text-sm opacity-80">
    One-line script<br/>
    Published on crates.io
  </div>
</div>

<div class="p-6 bg-red-900 bg-opacity-20 rounded-lg">
  <div class="text-4xl mb-2">🛡️</div>
  <div class="text-xl font-bold mb-2">Enterprise-Ready</div>
  <div class="text-sm opacity-80">
    Air-gapped environments<br/>
    52 pre-compiled safety patterns
  </div>
</div>

<div class="p-6 bg-yellow-900 bg-opacity-20 rounded-lg">
  <div class="text-4xl mb-2">🎯</div>
  <div class="text-xl font-bold mb-2">Focused Functionality</div>
  <div class="text-sm opacity-80">
    One thing exceptionally well:<br/>
    Terminal command assistance
  </div>
</div>

<div class="p-6 bg-orange-900 bg-opacity-20 rounded-lg">
  <div class="text-4xl mb-2">✅</div>
  <div class="text-xl font-bold mb-2">Safety-First</div>
  <div class="text-sm opacity-80">
    Independent validation layer<br/>
    Caught AI marking "rm -rf /" as Safe
  </div>
</div>

</v-clicks>

</div>

---
layout: center
---

# Market Opportunity

<div class="max-w-5xl mx-auto">

## Target Segments

<div class="grid grid-cols-3 gap-6 mt-8">

<v-clicks>

<div class="p-6 bg-blue-900 bg-opacity-20 rounded-lg">
  <div class="text-2xl font-bold mb-2">1. Enterprise Developers</div>
  <div class="text-sm opacity-80">
    Primary target<br/>
    Financial, healthcare, government<br/>
    Restricted environments
  </div>
</div>

<div class="p-6 bg-green-900 bg-opacity-20 rounded-lg">
  <div class="text-2xl font-bold mb-2">2. DevOps Engineers</div>
  <div class="text-sm opacity-80">
    Managing complex infrastructure<br/>
    Strict security protocols<br/>
    Automation workflows
  </div>
</div>

<div class="p-6 bg-purple-900 bg-opacity-20 rounded-lg">
  <div class="text-2xl font-bold mb-2">3. Security-Conscious Orgs</div>
  <div class="text-sm opacity-80">
    Air-gapped environments<br/>
    Limited connectivity<br/>
    Zero-trust architectures
  </div>
</div>

</v-clicks>

</div>

</div>

---
layout: center
---

# TAM/SAM/SOM Analysis

<div class="max-w-6xl mx-auto mt-8">

<div class="grid grid-cols-3 gap-8">

<v-click>
<div class="p-8 bg-gradient-to-br from-blue-900 to-blue-700 bg-opacity-30 rounded-lg">
  <div class="text-sm font-bold mb-2 opacity-80">Total Addressable Market</div>
  <div class="text-5xl font-bold mb-4">$3B</div>
  <div class="text-sm opacity-80">
    25M+ terminal users worldwide<br/>
    × $120/year
  </div>
</div>
</v-click>

<v-click>
<div class="p-8 bg-gradient-to-br from-green-900 to-green-700 bg-opacity-30 rounded-lg">
  <div class="text-sm font-bold mb-2 opacity-80">Serviceable Available Market</div>
  <div class="text-5xl font-bold mb-4">$1.44B</div>
  <div class="text-sm opacity-80">
    12M+ enterprise/restricted developers<br/>
    × $120/year
  </div>
</div>
</v-click>

<v-click>
<div class="p-8 bg-gradient-to-br from-purple-900 to-purple-700 bg-opacity-30 rounded-lg">
  <div class="text-sm font-bold mb-2 opacity-80">Serviceable Obtainable Market (3-year)</div>
  <div class="text-5xl font-bold mb-4">$12M</div>
  <div class="text-sm opacity-80">
    240K users<br/>
    × $50 average ARR
  </div>
</div>
</v-click>

</div>

</div>

---
layout: center
---

# Competitive Advantage

<div class="max-w-6xl mx-auto text-xs">

| Feature | **Caro** | GitHub Copilot CLI | Warp AI | Cursor Terminal | Aider |
|---------|----------|-------------------|---------|-----------------|-------|
| **Works offline** | ✅ | ❌ | ❌ | ❌ | Partial |
| **Enterprise security focus** | ✅ | Partial | ❌ | ❌ | ❌ |
| **No context switching** | ✅ | ❌ | ✅ | Partial | ❌ |
| **Minimal installation** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Rule-based guardrails** | ✅ | Partial | Partial | ❌ | ❌ |
| **GPU acceleration (local)** | ✅ | ❌ | ❌ | ❌ | Partial |
| **Open source (AGPL-3.0)** | ✅ | ❌ | ❌ | ❌ | Partial |
| **Published package** | ✅ | ✅ | ❌ | ❌ | ✅ |

</div>

<div class="mt-8 grid grid-cols-2 gap-4 max-w-4xl mx-auto text-sm">

<v-clicks>

<div class="p-4 bg-blue-900 bg-opacity-20 rounded">
  <strong>True Offline Operation</strong><br/>
  Only tool that bundles model with GPU acceleration
</div>

<div class="p-4 bg-green-900 bg-opacity-20 rounded">
  <strong>Safety Architecture</strong><br/>
  52 pre-compiled patterns operate outside AI model
</div>

<div class="p-4 bg-purple-900 bg-opacity-20 rounded">
  <strong>Enterprise Distribution</strong><br/>
  Single binary, air-gapped compatible
</div>

<div class="p-4 bg-orange-900 bg-opacity-20 rounded">
  <strong>Production Ready</strong><br/>
  v0.1.0 published with 44 library + 9 integration tests
</div>

</v-clicks>

</div>

---
layout: center
---

# Business Model

<div class="max-w-6xl mx-auto">

## Three-Tier Approach

<div class="grid grid-cols-3 gap-6 mt-8">

<v-click>
<div class="p-6 bg-gradient-to-br from-blue-900 to-blue-700 bg-opacity-30 rounded-lg">
  <div class="text-2xl font-bold mb-4">Community Edition</div>
  <div class="text-3xl font-bold mb-4 text-green-400">FREE</div>
  <ul class="text-sm space-y-2">
    <li>✓ Basic command assistance</li>
    <li>✓ Single local model (Qwen2.5-Coder-1.5B)</li>
    <li>✓ Community safety patterns</li>
    <li>✓ Open-source contributions</li>
  </ul>
</div>
</v-click>

<v-click>
<div class="p-6 bg-gradient-to-br from-green-900 to-green-700 bg-opacity-30 rounded-lg border-2 border-green-400">
  <div class="text-2xl font-bold mb-4">Pro Edition</div>
  <div class="text-3xl font-bold mb-4 text-green-400">$14.99<span class="text-lg">/mo</span></div>
  <ul class="text-sm space-y-2">
    <li>✓ Advanced command generation</li>
    <li>✓ Multiple model options</li>
    <li>✓ Custom aliases & workflows</li>
    <li>✓ Priority updates</li>
    <li>✓ Commercial license</li>
  </ul>
</div>
</v-click>

<v-click>
<div class="p-6 bg-gradient-to-br from-purple-900 to-purple-700 bg-opacity-30 rounded-lg">
  <div class="text-2xl font-bold mb-4">Enterprise Edition</div>
  <div class="text-3xl font-bold mb-4 text-purple-400">$79<span class="text-lg">/seat/mo</span></div>
  <ul class="text-sm space-y-2">
    <li>✓ Everything in Pro, plus:</li>
    <li>✓ Custom deployment options</li>
    <li>✓ Compliance reporting</li>
    <li>✓ Custom guardrails & policies</li>
    <li>✓ SSO/SAML integration</li>
    <li>✓ Dedicated support with SLAs</li>
  </ul>
  <div class="text-xs mt-2 opacity-60">Minimum 50 seats</div>
</div>
</v-click>

</div>

</div>

---
layout: center
---

# Enterprise Integration Strategy

<div class="max-w-5xl mx-auto">

## Addressing Enterprise Barriers

<div class="grid grid-cols-2 gap-6 mt-8">

<v-clicks>

<div class="p-6 bg-blue-900 bg-opacity-20 rounded-lg">
  <div class="text-xl font-bold mb-3">🏢 Vendor Management</div>
  <ul class="text-sm space-y-2">
    <li>✓ Published on crates.io (official Rust registry)</li>
    <li>✓ Single approval process</li>
    <li>✓ Existing procurement workflows</li>
    <li>✓ No external dependencies</li>
  </ul>
</div>

<div class="p-6 bg-green-900 bg-opacity-20 rounded-lg">
  <div class="text-xl font-bold mb-3">🔐 Security Compliance</div>
  <ul class="text-sm space-y-2">
    <li>✓ Rule-based guardrails independent of model</li>
    <li>✓ No data leaves environment</li>
    <li>✓ Comprehensive audit logging</li>
    <li>✓ SHA256 integrity validation</li>
  </ul>
</div>

<div class="p-6 bg-purple-900 bg-opacity-20 rounded-lg">
  <div class="text-xl font-bold mb-3">🖥️ IT Restrictions</div>
  <ul class="text-sm space-y-2">
    <li>✓ Works with approved distributions</li>
    <li>✓ Minimal system requirements</li>
    <li>✓ No cloud connectivity required</li>
    <li>✓ Single binary distribution</li>
  </ul>
</div>

<div class="p-6 bg-orange-900 bg-opacity-20 rounded-lg">
  <div class="text-xl font-bold mb-3">🎯 Integration Examples</div>
  <ul class="text-sm space-y-2">
    <li>💰 Financial: Trading environments</li>
    <li>🏥 Healthcare: HIPAA-compliant</li>
    <li>🏛️ Government: Classified dev environments</li>
  </ul>
</div>

</v-clicks>

</div>

</div>

---
layout: center
---

# Current Status & Traction

<div class="max-w-6xl mx-auto">

<div class="grid grid-cols-2 gap-8">

<div>

## Product ✅

<v-clicks>

- ✅ **v0.1.0** published on crates.io
- ✅ **Website** live at caro.sh
- ✅ **MLX backend** fully operational with Metal GPU
- ✅ **Qwen2.5-Coder-1.5B** production model (87% accuracy)
- ✅ **52 safety patterns** pre-compiled
- ✅ **One-line install** script
- ✅ **Comprehensive testing** (44 lib + 9 integration)

</v-clicks>

</div>

<div>

## Technical Milestones ✅

<v-clicks>

- ✅ Multi-backend architecture (MLX, CPU, remote)
- ✅ Configuration management system
- ✅ Safety validation with risk assessment
- ✅ Interactive user confirmation flows
- ✅ Multiple output formats (JSON, YAML, Plain)
- ✅ Cross-platform support (macOS, Linux, Windows)
- ✅ Spec-kitty integration (rapid development)
- ✅ GitHub Actions CI/CD pipeline

</v-clicks>

</div>

</div>

<div v-click class="mt-8 p-6 bg-green-900 bg-opacity-30 rounded-lg text-center">
  <div class="text-2xl font-bold">
    We're not pre-product. We're scaling a working solution. 🚀
  </div>
</div>

</div>

---
layout: center
---

# 2026 Roadmap

<div class="max-w-6xl mx-auto text-sm">

<div class="grid grid-cols-3 gap-4">

<v-click>
<div class="p-4 bg-blue-900 bg-opacity-20 rounded-lg">
  <div class="text-lg font-bold mb-2">v1.1.0 - Core Improvements</div>
  <div class="text-xs opacity-60 mb-3">February 15, 2026</div>
  <ul class="text-xs space-y-1">
    <li>• Hugging Face model download</li>
    <li>• Performance optimization</li>
    <li>• Safety enhancements</li>
    <li>• LLM evaluation harness</li>
    <li>• Benchmark suite</li>
  </ul>
</div>
</v-click>

<v-click>
<div class="p-4 bg-green-900 bg-opacity-20 rounded-lg">
  <div class="text-lg font-bold mb-2">v1.2.0 - Website & Docs</div>
  <div class="text-xs opacity-60 mb-3">March 31, 2026</div>
  <ul class="text-xs space-y-1">
    <li>• Strategic product roadmap</li>
    <li>• Website value proposition</li>
    <li>• Astro Starlight docs site</li>
    <li>• Interactive terminal landing</li>
    <li>• Marketing materials</li>
  </ul>
</div>
</v-click>

<v-click>
<div class="p-4 bg-purple-900 bg-opacity-20 rounded-lg">
  <div class="text-lg font-bold mb-2">v2.0.0 - Advanced Features</div>
  <div class="text-xs opacity-60 mb-3">June 30, 2026</div>
  <ul class="text-xs space-y-1">
    <li>• Karo distributed intelligence</li>
    <li>• Dogma rule engine</li>
    <li>• Voice synthesis for Caro</li>
    <li>• Self-healing features</li>
    <li>• Local context indexing</li>
  </ul>
</div>
</v-click>

</div>

<v-click>
<div class="mt-6 p-4 bg-orange-900 bg-opacity-20 rounded-lg">
  <div class="text-lg font-bold mb-2">Long-term (12-18 months)</div>
  <div class="grid grid-cols-2 gap-4 text-xs mt-3">
    <div>
      <strong>Enterprise Readiness:</strong>
      <ul class="mt-1 space-y-1">
        <li>• SOC2 Type II compliance</li>
        <li>• FedRAMP authorization</li>
        <li>• HIPAA compliance validation</li>
        <li>• Enterprise admin dashboard</li>
      </ul>
    </div>
    <div>
      <strong>Platform Expansion:</strong>
      <ul class="mt-1 space-y-1">
        <li>• MCP server integration</li>
        <li>• Claude Desktop skill</li>
        <li>• VS Code extension</li>
        <li>• JetBrains IDE plugin</li>
      </ul>
    </div>
  </div>
</div>
</v-click>

</div>

---
layout: center
class: text-center
---

# The Ask

<div class="text-5xl font-bold mb-8 text-blue-400">
  $2.5M Seed Investment
</div>

<div class="text-xl mb-8">
  To accelerate market adoption and achieve enterprise readiness
</div>

---
layout: center
---

# Use of Funds

<div class="max-w-6xl mx-auto">

<div class="grid grid-cols-2 gap-6">

<v-click>
<div class="p-6 bg-blue-900 bg-opacity-20 rounded-lg">
  <div class="text-2xl font-bold mb-2">Engineering & Product</div>
  <div class="text-3xl font-bold text-blue-400 mb-3">55% - $1.375M</div>
  <ul class="text-sm space-y-1">
    <li>• 3 senior Rust engineers ($450K)</li>
    <li>• ML/AI engineer ($175K)</li>
    <li>• Security engineer ($175K)</li>
    <li>• Infrastructure & tooling ($75K)</li>
    <li>• Model fine-tuning & compute ($200K)</li>
    <li>• Contract development ($300K)</li>
  </ul>
</div>
</v-click>

<v-click>
<div class="p-6 bg-green-900 bg-opacity-20 rounded-lg">
  <div class="text-2xl font-bold mb-2">Sales & Marketing</div>
  <div class="text-3xl font-bold text-green-400 mb-3">25% - $625K</div>
  <ul class="text-sm space-y-1">
    <li>• VP Sales / Head of GTM ($200K)</li>
    <li>• 2 Enterprise AEs ($250K)</li>
    <li>• Marketing & content ($100K)</li>
    <li>• Events & conferences ($50K)</li>
    <li>• Demand generation ($25K)</li>
  </ul>
</div>
</v-click>

<v-click>
<div class="p-6 bg-purple-900 bg-opacity-20 rounded-lg">
  <div class="text-2xl font-bold mb-2">Compliance & Legal</div>
  <div class="text-3xl font-bold text-purple-400 mb-3">12% - $300K</div>
  <ul class="text-sm space-y-1">
    <li>• SOC2 Type II certification ($100K)</li>
    <li>• FedRAMP authorization prep ($150K)</li>
    <li>• Legal (contracts, licensing, IP) ($50K)</li>
  </ul>
</div>
</v-click>

<v-click>
<div class="p-6 bg-orange-900 bg-opacity-20 rounded-lg">
  <div class="text-2xl font-bold mb-2">Operations & Runway</div>
  <div class="text-3xl font-bold text-orange-400 mb-3">8% - $200K</div>
  <ul class="text-sm space-y-1">
    <li>• Finance & accounting ($50K)</li>
    <li>• HR & recruiting ($30K)</li>
    <li>• Insurance & admin ($40K)</li>
    <li>• Buffer ($80K)</li>
  </ul>
</div>
</v-click>

</div>

</div>

---
layout: center
---

# 18-Month Milestones

<div class="max-w-6xl mx-auto">

<div class="grid grid-cols-3 gap-6">

<v-click>
<div class="p-6 bg-gradient-to-br from-blue-900 to-blue-700 bg-opacity-30 rounded-lg">
  <div class="text-lg font-bold mb-3">Month 6</div>
  <ul class="text-sm space-y-2">
    <li>👥 Team: 6 engineers + 2 GTM</li>
    <li>📊 5,000 MAU</li>
    <li>💰 100 Pro subscribers</li>
    <li>✅ SOC2 Type I complete</li>
    <li>🤝 3 enterprise pilots signed</li>
  </ul>
</div>
</v-click>

<v-click>
<div class="p-6 bg-gradient-to-br from-green-900 to-green-700 bg-opacity-30 rounded-lg">
  <div class="text-lg font-bold mb-3">Month 12</div>
  <ul class="text-sm space-y-2">
    <li>📊 10,000 MAU</li>
    <li>💰 300 Pro subscribers</li>
    <li>✅ SOC2 Type II certified</li>
    <li>🏢 10 enterprise customers</li>
    <li>💵 <strong>$500K ARR</strong></li>
  </ul>
  <div class="text-xs mt-2 opacity-60">Avg $40K ACV</div>
</div>
</v-click>

<v-click>
<div class="p-6 bg-gradient-to-br from-purple-900 to-purple-700 bg-opacity-30 rounded-lg border-2 border-purple-400">
  <div class="text-lg font-bold mb-3">Month 18</div>
  <ul class="text-sm space-y-2">
    <li>📊 25,000 MAU</li>
    <li>💰 500 Pro subscribers</li>
    <li>🏢 30 enterprise customers</li>
    <li>🔐 FedRAMP in progress</li>
    <li>💵 <strong>$2M ARR</strong></li>
    <li>🚀 <strong>Series A ready</strong></li>
  </ul>
  <div class="text-xs mt-2 opacity-60">Avg $50K ACV</div>
</div>
</v-click>

</div>

</div>

---
layout: center
---

# Why Now?

<div class="max-w-5xl mx-auto">

<div class="grid grid-cols-2 gap-6">

<v-clicks>

<div class="p-6 bg-blue-900 bg-opacity-20 rounded-lg">
  <div class="text-3xl mb-2">📈</div>
  <div class="text-lg font-bold mb-2">AI Adoption Accelerating</div>
  <div class="text-sm">
    Enterprise AI spend: <strong>$50B in 2024</strong><br/>
    Projected: <strong>$150B by 2027</strong>
  </div>
</div>

<div class="p-6 bg-red-900 bg-opacity-20 rounded-lg">
  <div class="text-3xl mb-2">🔒</div>
  <div class="text-lg font-bold mb-2">Security Concerns Persist</div>
  <div class="text-sm">
    <strong>68%</strong> of enterprises cite<br/>
    data sovereignty concerns
  </div>
</div>

<div class="p-6 bg-green-900 bg-opacity-20 rounded-lg">
  <div class="text-3xl mb-2">⚡</div>
  <div class="text-lg font-bold mb-2">Developer Productivity Arms Race</div>
  <div class="text-sm">
    <strong>12% YoY</strong> growth in tool spend<br/>
    Competition for best talent
  </div>
</div>

<div class="p-6 bg-purple-900 bg-opacity-20 rounded-lg">
  <div class="text-3xl mb-2">💻</div>
  <div class="text-lg font-bold mb-2">Terminal Remains Essential</div>
  <div class="text-sm">
    <strong>78%</strong> of developers use terminal daily<br/>
    (vs 71% in 2020)
  </div>
</div>

<div class="p-6 bg-yellow-900 bg-opacity-20 rounded-lg">
  <div class="text-3xl mb-2">🤖</div>
  <div class="text-lg font-bold mb-2">Local AI is Production-Ready</div>
  <div class="text-sm">
    1.5B parameter models achieve<br/>
    <strong>87% task accuracy</strong>
  </div>
</div>

<div class="p-6 bg-orange-900 bg-opacity-20 rounded-lg">
  <div class="text-3xl mb-2">⚖️</div>
  <div class="text-lg font-bold mb-2">Regulatory Tailwinds</div>
  <div class="text-sm">
    GDPR, CCPA, EU AI Act<br/>
    driving on-premise requirements
  </div>
</div>

</v-clicks>

</div>

</div>

---
layout: center
class: text-center
---

# Category Creator Position

<div class="max-w-4xl mx-auto mt-8">

<v-clicks>

<div class="text-2xl mb-8">
  We have <strong class="text-blue-400">12-18 months</strong> to own<br/>
  <strong>"offline enterprise AI tools"</strong><br/>
  before large incumbents realize the opportunity
</div>

<div class="grid grid-cols-2 gap-8">

<div class="p-6 bg-red-900 bg-opacity-20 rounded-lg">
  <div class="text-lg font-bold mb-2">Why Incumbents Can't Compete</div>
  <ul class="text-sm text-left space-y-2">
    <li>☁️ Cloud providers: structurally misaligned</li>
    <li>🔐 GitHub: tied to cloud identity</li>
    <li>🏢 Enterprise vendors: slow to innovate</li>
  </ul>
</div>

<div class="p-6 bg-green-900 bg-opacity-20 rounded-lg">
  <div class="text-lg font-bold mb-2">Our Advantages</div>
  <ul class="text-sm text-left space-y-2">
    <li>🎯 Purpose-built for offline</li>
    <li>⚡ First-mover in category</li>
    <li>🛡️ Safety-first architecture</li>
  </ul>
</div>

</div>

</v-clicks>

</div>

---
layout: center
class: text-center
---

# Closing

<div class="max-w-4xl mx-auto mt-8 text-xl leading-relaxed">

<v-clicks>

<div class="mb-8">
  Caro represents the <strong class="text-blue-400">future of enterprise AI tools</strong>
</div>

<div class="mb-8">
  Secure • Transparent • Respecting organizational boundaries<br/>
  while delivering <strong class="text-green-400">genuine productivity gains</strong>
</div>

<div class="mb-8">
  We're proving AI can work <strong>within enterprise security models</strong><br/>
  without compromise
</div>

<div class="p-6 bg-gradient-to-r from-blue-900 to-purple-900 bg-opacity-30 rounded-lg">
  The question isn't <em>whether</em> enterprises will adopt AI for developer productivity<br/>
  <div class="mt-4 text-2xl font-bold text-blue-400">
    The question is: Will they use tools that compromise security,<br/>
    or tools like Caro purpose-built for their requirements?
  </div>
</div>

</v-clicks>

</div>

---
layout: center
class: text-center
---

# Thank You

<div class="mt-8 text-xl">
  Questions?
</div>

<div class="mt-12 grid grid-cols-3 gap-8 max-w-4xl mx-auto">

<div>
  <div class="text-sm opacity-60">Website</div>
  <div class="text-lg font-bold">caro.sh</div>
</div>

<div>
  <div class="text-sm opacity-60">GitHub</div>
  <div class="text-lg font-bold">github.com/wildcard/caro</div>
</div>

<div>
  <div class="text-sm opacity-60">Published</div>
  <div class="text-lg font-bold">crates.io/crates/caro</div>
</div>

</div>

<div class="absolute bottom-10 left-0 right-0 text-center">
  <div class="text-sm opacity-50">
    Caro v0.1.0 • Production Ready • December 2024
  </div>
</div>

---
layout: end
---

# Appendix

Additional materials and data

---

# Key Messaging Points

<div class="max-w-5xl mx-auto text-sm">

## Always Emphasize:

<v-clicks>

1. **Not pre-product** - v0.1.0 published, working GPU acceleration, real users

2. **Safety is differentiator** - AI models can't be trusted alone, our guardrails saved users from `rm -rf /`

3. **Enterprise is the unlock** - $1.4B enterprise opportunity

4. **Offline is the moat** - True offline operation creates 12+ month competitive delay

5. **Community creates defensibility** - Open source safety patterns build network effects

6. **Timing is critical** - 12-18 month window before incumbents recognize category

</v-clicks>

</div>

<v-click>

<div class="mt-8 p-6 bg-blue-900 bg-opacity-30 rounded-lg max-w-5xl mx-auto">
  <div class="font-bold mb-2">Value proposition in one sentence:</div>
  <div class="text-lg">
    "Caro is the only production-ready AI shell assistant that works in air-gapped enterprise environments, with independent safety validation and GPU acceleration - serving the 8 million developers who can't use cloud-based AI tools."
  </div>
</div>

</v-click>
