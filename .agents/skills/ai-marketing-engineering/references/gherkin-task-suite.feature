@orchestrator @task_suites @ai_marketing_engineering
Feature: AI Marketing Engineering Task Suite (Alon Huri Persona)

  Background:
    Given the Persona Spec for "Alon Huri" is loaded as authoritative
    And the voice is: direct, technical, evidence-driven, pragmatic
    And the quality bar requires: measurable outcomes, concrete implementations, B2C/B2B distinction
    And outputs must NOT include confidential startup details
    And all marketing automation must be value-first (no spam)

  # ===========================================================================
  # OBJECTIVE DAILY WORK TASKS (5 scenarios)
  # "Then" clauses are verifiable: counts, statuses, explicit artifacts, thresholds
  # ===========================================================================

  @objective @daily @creative @meta
  Scenario: Generate and classify Meta ad creative variations
    Given a base creative brief with 3 images, 5 headlines, and 4 CTAs exists
    And the campaign objective is "conversion" for B2C e-commerce
    When I generate creative variations using combinatorial expansion
    Then I produce exactly 60 variations (3 x 5 x 4)
    And each variation is tagged with:
      | field          | requirement           |
      | variation_id   | unique identifier     |
      | parent_id      | null for base         |
      | changed_axes   | list of what changed  |
      | test_priority  | 1-5 scale             |
    And I output a CSV file with columns: id, image_id, headline, cta, priority
    And I produce a test plan with 3 cohorts of 20 variations each

  @objective @daily @budget @performance
  Scenario: Execute adaptive budget reallocation based on CPL thresholds
    Given 10 active campaigns with current budgets and 7-day performance data
    And the rule set includes:
      | trigger                | action                  | cap      |
      | CPL < target x 0.8     | increase_budget_15%     | 40% max  |
      | CPL > target x 1.3     | decrease_budget_20%     | $100 min |
      | ROAS > 4               | increase_budget_10%     | 30% max  |
    When I apply the adaptive budget rules
    Then every campaign has a recommendation of: "increase", "decrease", or "hold"
    And total recommended budget equals original total budget (zero-sum)
    And I produce a change log with:
      | field       | requirement                |
      | campaign_id | matches input              |
      | old_budget  | current value              |
      | new_budget  | after rules               |
      | change_pct  | calculated percentage      |
      | rule_fired  | which rule triggered       |
    And no single campaign exceeds 40% of total budget

  @objective @daily @signals @analytics
  Scenario: Identify top 5 LTV signal correlations from cohort data
    Given a dataset with 10,000 users, 50 behavioral features, and 6-month LTV outcomes
    When I run correlation analysis with minimum confidence threshold of 95%
    Then I output exactly 5 findings ranked by "surprising_level" (counterintuitive first)
    And each finding includes:
      | field                   | requirement               |
      | pattern                 | natural language description |
      | correlation_strength    | r² or odds ratio           |
      | confidence_interval     | 95% CI bounds              |
      | sample_size             | n supporting the pattern   |
      | segment                 | "all" or specific segment  |
      | validation_experiment   | hypothesis + test design   |
    And I flag any finding where segment size < 500 as "low power"

  @objective @daily @aeo @monitoring
  Scenario: Audit brand mentions in LLM answer engines
    Given a list of 10 target queries and 3 LLMs (ChatGPT, Perplexity, Claude)
    When I query each LLM with each target query
    Then I produce a mention matrix (10 x 3) with values: "mentioned", "cited", "absent"
    And I calculate brand share of voice: (mentioned + cited) / total queries per LLM
    And I identify top 3 queries where competitors are cited but we are not
    And I output an action list with:
      | query              | gap_type          | recommended_content   |
      | [specific query]   | absent/competitor | [content suggestion]  |

  @objective @daily @churn @support
  Scenario: Classify support tickets by churn risk and assign interventions
    Given 50 new support tickets from the last 24 hours with text and metadata
    When I analyze sentiment and urgency signals
    Then every ticket is classified into:
      | risk_level | criteria                                | intervention        |
      | critical   | sentiment < -0.5 AND urgency >= 4       | immediate_escalation|
      | high       | sentiment < -0.3 OR churn_keywords      | proactive_outreach  |
      | medium     | sentiment < 0 AND repeat_ticket         | empathy_response    |
      | low        | sentiment >= 0                          | standard_response   |
    And I produce a summary report:
      | metric                    | value         |
      | total_tickets             | 50            |
      | critical_count            | [count]       |
      | high_count                | [count]       |
      | escalation_queue_size     | [critical count] |
    And each critical ticket has owner assigned within 1 hour SLA


  # ===========================================================================
  # SUBJECTIVE WORK TASKS (10 scenarios)
  # Each includes: rubric + scoring, 2+ alternatives, review/revision loop
  # ===========================================================================

  @subjective @strategy @creative @writing
  Scenario: Design a creative evolution strategy memo for a B2C startup
    Given a B2C startup spending $50K/month on Meta ads with flat performance
    And the audience is the founding team + first marketing hire
    When I draft a "Creative Evolution Strategy" memo
    Then I include:
      | section                    |
      | Executive Summary          |
      | Current State Analysis     |
      | Proposed Mechanism         |
      | Implementation Roadmap     |
      | Resource Requirements      |
      | Success Metrics            |
      | Risks and Mitigations      |
    And I score it with a rubric:
      | criterion              | scale | weight |
      | clarity                | 1-5   | 20%    |
      | actionability          | 1-5   | 30%    |
      | technical feasibility  | 1-5   | 25%    |
      | voice_fidelity         | 1-5   | 15%    |
      | measurable_outcomes    | 1-5   | 10%    |
    And I provide 2 alternatives:
      | version          | description                                   |
      | aggressive       | 3-week implementation, higher risk, faster ROI|
      | conservative     | 8-week implementation, lower risk, proven path|
    And I revise once focusing on the lowest-scoring criterion
    And I include explicit B2C applicability note (not applicable to B2B enterprise)

  @subjective @architecture @data @writing
  Scenario: Draft a data layer architecture RFC for AI agent consumption
    Given the marketing team uses 5 data sources (GA4, Mixpanel, Stripe, Hubspot, BigQuery)
    And the goal is to enable AI agents to query marketing data conversationally
    When I draft the RFC
    Then I include:
      | section                   |
      | Problem Statement         |
      | Proposed Architecture     |
      | Data Model                |
      | Query Interface Design    |
      | Security & Permissions    |
      | Implementation Phases     |
      | Success Criteria          |
    And I provide 2 alternatives:
      | version       | description                                    |
      | lightweight   | Single semantic layer, 2-week build            |
      | comprehensive | Full data mesh, 6-week build, more flexibility |
    And I run a review loop:
      | review_step                           |
      | identify weakest section              |
      | check for missing failure modes       |
      | revise RFC once                       |
    And I score with rubric:
      | criterion              | scale |
      | technical_depth        | 1-5   |
      | operational_clarity    | 1-5   |
      | security_coverage      | 1-5   |

  @subjective @aeo @content @strategy
  Scenario: Create an AEO content strategy for a SaaS product
    Given a B2B SaaS product in the project management space
    And competitors are mentioned by ChatGPT and Perplexity but we are not
    When I develop an AEO strategy document
    Then it includes:
      | section                              |
      | Current LLM Visibility Audit         |
      | Target Query Map (20 queries)        |
      | Content Gap Analysis                 |
      | Community Engagement Plan (Reddit)   |
      | Content Calendar (8 weeks)           |
      | Monitoring & Iteration Cadence       |
    And I provide 2 tone variants:
      | variant          | description                              |
      | thought_leader   | Original research, contrarian takes      |
      | practical_guide  | How-to content, step-by-step tutorials   |
    And I score with rubric:
      | criterion              | scale |
      | query_relevance        | 1-5   |
      | community_authenticity | 1-5   |
      | measurability          | 1-5   |
    And I revise once based on the weakest rubric criterion
    And I note B2B vs B2C applicability (this is B2B focused)

  @subjective @quiz @conversion @design
  Scenario: Design a dynamic qualification quiz for a fintech product
    Given a fintech product with 3 customer segments (SMB, Mid-market, Enterprise)
    And conversion goal is to route users to: self-serve trial, sales demo, or nurture
    When I design the quiz flow
    Then I produce:
      | deliverable                    |
      | Decision tree (visual)         |
      | Question bank (15+ questions)  |
      | Branching logic rules          |
      | Segment-to-handoff mapping     |
      | Personalization triggers       |
    And I provide 2 alternatives:
      | version    | description                                |
      | minimal    | 5 questions, fast completion, basic routing|
      | deep       | 10 questions, rich personalization, precise routing |
    And I run a review loop:
      | check                                    |
      | verify no dead-end paths                 |
      | ensure skip option always available      |
      | revise once for lowest-scoring criterion |
    And I score with rubric:
      | criterion              | scale |
      | user_experience        | 1-5   |
      | qualification_accuracy | 1-5   |
      | personalization_depth  | 1-5   |

  @subjective @activation @product @analysis
  Scenario: Develop a friction detection and intervention framework
    Given product telemetry showing 40% drop-off between signup and first value action
    And the "aha moment" is defined as "user creates first project and invites 1 teammate"
    When I develop a friction detection framework
    Then it includes:
      | component                           |
      | Friction signal definitions         |
      | Intervention trigger rules          |
      | Intervention templates (5+)         |
      | A/B test plan                       |
      | Success metrics dashboard spec      |
    And I provide 2 alternatives:
      | version      | description                                   |
      | reactive     | Trigger after friction detected               |
      | proactive    | Predict friction, intervene before drop-off   |
    And I score with rubric:
      | criterion                | scale |
      | signal_specificity       | 1-5   |
      | intervention_relevance   | 1-5   |
      | measurable_impact        | 1-5   |
    And I revise once focusing on the lowest-scoring criterion

  @subjective @video @personalization @campaign
  Scenario: Plan a personalized video outreach campaign for enterprise prospects
    Given a list of 200 enterprise prospects with name, company, and role
    And a base video template with 3 personalization points (name, company, pain point)
    When I create a campaign plan
    Then it includes:
      | section                        |
      | Video production pipeline      |
      | Quality assurance checklist    |
      | Personalization rules          |
      | Delivery sequence              |
      | Response handling playbook     |
      | Success metrics                |
    And I provide 2 alternatives:
      | version       | description                                  |
      | high_touch    | Human QA on all videos, slower, premium feel |
      | scaled        | Automated QA, faster, some quality variance  |
    And I run a review loop:
      | check                                 |
      | verify consent requirements met       |
      | check for uncanny valley risks        |
      | revise once                           |
    And I score with rubric:
      | criterion              | scale |
      | personalization_quality| 1-5   |
      | brand_consistency      | 1-5   |
      | scalability            | 1-5   |

  @subjective @competitive @content @marketing
  Scenario: Create competitor weakness landing page strategy
    Given 3 main competitors with 500+ reviews each on G2 and Capterra
    And our product differentiators are: better UX, faster onboarding, lower price
    When I develop a competitive landing page strategy
    Then it includes:
      | deliverable                          |
      | Pain point taxonomy (20+ pain points)|
      | Landing page briefs (10 pages)       |
      | SEO keyword mapping                  |
      | Content calendar                     |
      | Competitive positioning matrix       |
    And I provide 2 alternatives:
      | version       | description                                     |
      | aggressive    | Direct competitor naming, side-by-side comparison|
      | subtle        | Pain-point focused, no competitor names         |
    And I score with rubric:
      | criterion              | scale |
      | pain_point_accuracy    | 1-5   |
      | differentiator_clarity | 1-5   |
      | ethical_compliance     | 1-5   |
    And I revise once based on the weakest rubric criterion
    And I verify no false claims about competitors

  @subjective @churn @customer_success @playbook
  Scenario: Design a churn prevention intervention playbook
    Given churn rate is 5% monthly and support ticket sentiment correlates with churn
    And available intervention channels: email, in-app, phone, discount offers
    When I create an intervention playbook
    Then it includes:
      | section                             |
      | Risk signal taxonomy                |
      | Intervention decision tree          |
      | Response templates (10+)            |
      | Escalation criteria                 |
      | Compensation authority levels       |
      | Success metrics                     |
    And I provide 2 alternatives:
      | version       | description                                |
      | automated     | AI-driven, 90% automated, fast response    |
      | human_hybrid  | AI triage + human intervention, higher touch|
    And I run a review loop:
      | check                                     |
      | ensure no manipulation of genuine issues  |
      | verify legal compliance for offers        |
      | revise once                               |
    And I score with rubric:
      | criterion              | scale |
      | empathy_quality        | 1-5   |
      | escalation_coverage    | 1-5   |
      | legal_compliance       | 1-5   |

  @subjective @prompting @architecture @master_prompt
  Scenario: Compile a persona spec into a deployable master prompt
    Given a Persona Spec with: identity, audience, voice, constraints, quality bar
    And the target deployment is a multi-agent marketing automation system
    When I compile it into a Master Prompt
    Then the Master Prompt includes:
      | component                  |
      | Shared invariants          |
      | Task router logic          |
      | Agent cards (10 agents)    |
      | Synthesis rules            |
      | Quality gates              |
    And I score it with a rubric:
      | criterion              | scale |
      | voice_fidelity         | 1-5   |
      | routing_coverage       | 1-5   |
      | operational_clarity    | 1-5   |
      | constraint_enforcement | 1-5   |
    And I provide 2 alternatives:
      | version     | description                                   |
      | strict      | Narrow agent boundaries, explicit handoffs    |
      | flexible    | Overlapping agent domains, emergent routing   |
    And I revise once focusing on the lowest-scoring criterion

  @subjective @hiring @strategy @communication
  Scenario: Draft a "Marketing Co-Founder" job brief instead of VP Marketing
    Given a seed-stage B2C startup with technical founders
    And the need is for someone who can "build the marketing machine, not just write copy"
    When I draft the job brief
    Then it includes:
      | section                         |
      | Why not VP Marketing?           |
      | What we're really looking for   |
      | Required skills                 |
      | Nice-to-have skills             |
      | What you'll build (mechanisms)  |
      | Equity/compensation philosophy  |
      | Interview process               |
    And I provide 2 tone alternatives:
      | version       | description                              |
      | provocative   | Challenge conventional hiring, bold tone |
      | professional  | Standard JD format, softer positioning   |
    And I score with rubric:
      | criterion              | scale |
      | clarity_of_need        | 1-5   |
      | differentiation        | 1-5   |
      | attractiveness         | 1-5   |
    And I run a review loop:
      | check                                  |
      | verify no discriminatory language      |
      | ensure realistic expectations          |
      | revise once                            |
    And I note this is based on Alon Huri's philosophy: "bring a marketing co-founder, not a VP"


  # ===========================================================================
  # FACTORY TEMPLATES (optional quick generators)
  # ===========================================================================

  @factory
  Scenario Outline: Create an objective marketing engineering task
    Given the mechanism is "<mechanism>"
    And the daily objective is "<objective>"
    When I perform "<action>"
    Then I produce "<artifact>"
    And "<metric>" meets threshold "<threshold>"

    Examples:
      | mechanism        | objective              | action                    | artifact                  | metric              | threshold |
      | creative         | refresh ad variations  | generate 50 new variants  | variation_batch.csv       | variants_count      | 50        |
      | budget           | optimize spend         | apply reallocation rules  | budget_changes.json       | total_budget_delta  | 0         |
      | signals          | find LTV predictors    | run correlation analysis  | findings_report.md        | findings_count      | 5         |
      | aeo              | monitor LLM presence   | query 3 LLMs x 10 queries | mention_matrix.csv        | coverage_percentage | 30%       |
      | churn            | classify support risk  | analyze 50 tickets        | risk_classification.json  | critical_flagged    | >0        |

  @factory
  Scenario Outline: Create a subjective marketing engineering task
    Given the audience is "<audience>"
    And the subjective goal is "<goal>"
    When I draft "<artifact>"
    Then I score it with a rubric:
      | criterion        | scale |
      | voice_fidelity   | 1-5   |
      | actionability    | 1-5   |
      | constraint_fit   | 1-5   |
    And I provide 2 alternatives: "<alt1>" and "<alt2>"
    And I revise once based on the weakest rubric criterion

    Examples:
      | audience    | goal                        | artifact              | alt1                | alt2                  |
      | founders    | explain creative evolution  | strategy_memo.md      | 1-pager summary     | detailed RFC          |
      | marketing   | design AEO strategy         | aeo_playbook.md       | community-first     | content-first         |
      | engineering | spec data layer             | architecture_rfc.md   | lightweight MVP     | comprehensive design  |
      | execs       | justify marketing headcount | business_case.md      | ROI-focused         | capability-focused    |
