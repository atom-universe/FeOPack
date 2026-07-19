# Personal Systems Engineer Spec

Status: living document
Owner: carbon
Created: 2026-07-13

## Purpose

This document records a long-term professional direction, the reasoning behind it, and how the direction changes over time.

The goal is not to create a motivational roadmap. The goal is to keep a durable knowledge base that can be reviewed every few months:

- What direction am I aiming at?
- Why does this direction fit my current assets?
- What assumptions are behind the plan?
- What evidence would prove the plan is working?
- What evidence would tell me to adjust?

## Current Positioning

The strongest current positioning is:

> Programming Systems Engineer, growing toward AI-era Developer Infrastructure / Systems Engineering.

This is different from both "frontend engineer" and "AI algorithm engineer".

The current asset base is closer to:

- Web architecture and frontend engineering experience.
- Frontend infrastructure and tooling interest.
- Bundler, loader pipeline, compiler, runtime, and Rust study.
- Interest in "why the system is designed this way", not only "how to use the framework".
- Practical engineering background from large-company work.

The recommended move is not a hard career reset. It is a layer shift:

```text
Frontend / Web Architecture
  -> Frontend Infrastructure
  -> Build Tools / Compiler / Runtime
  -> Programming Systems
  -> AI-native Developer Infrastructure
```

## Long-Term Target

The 10-year target is:

> Become someone who can create or lead core designs in developer tools, runtime systems, compilers, or AI infrastructure.

More concrete versions:

- Principal Systems Engineer.
- Research Engineer.
- Developer Infrastructure Architect.
- AI-native Developer Tooling / Runtime Engineer.
- Compiler / Build System / Programming Systems specialist.

The target is not:

- Pure application-layer frontend architect forever.
- Trend-following AI product engineer.
- Direct leap into frontier model algorithm research as the primary plan.

Pure model algorithm research is not impossible, but it is not the highest expected-value path given the current background and competitive landscape. The better route is to reuse existing engineering assets and move downward into systems.

## Core Thesis

AI increases the value of people who can design the systems that developers and AI agents depend on.

Models, prompts, and product patterns change quickly. The durable layers are:

- Runtime.
- Compiler.
- Scheduler.
- Memory model.
- Storage.
- Distributed system.
- Developer workflow infrastructure.
- Verification and observability.
- Interfaces between AI agents and real engineering environments.

The most valuable direction is therefore not "use AI to write more code", but:

> Build the infrastructure that makes human and AI software development reliable, inspectable, fast, and scalable.

## Strategic Bet

The highest expected-value bet is:

> Combine frontend/tooling background with systems training and AI-native workflow design.

This creates a rarer profile than a normal frontend engineer, and a more realistic path than competing directly with top-tier algorithm researchers.

The target skill stack:

- TypeScript and web platform expertise.
- Rust or C++ systems implementation ability.
- Compiler and build-system fundamentals.
- Operating system and runtime fundamentals.
- Performance profiling and observability.
- AI-assisted development workflow design.
- Strong technical writing and public artifacts.

## Five-Year Roadmap

### Phase 1: 2026-2027, Top-Tier Tooling Engineer

Goal:

> Become a strong tooling engineer with credible systems foundations.

Focus areas:

- Bundler architecture.
- Loader and plugin systems.
- Module graph and dependency graph.
- Incremental build and cache invalidation.
- Source maps and HMR.
- JS/Rust bridge.
- TypeScript, SWC, Oxc, Rspack, Vite, Rolldown-style ecosystems.

Expected outputs:

- A serious Feopack or adjacent tooling project.
- 5-10 non-trivial technical notes.
- Several meaningful PRs or design documents.
- A clear source-reading trail for Rspack, SWC, Oxc, Vite, or similar systems.

Validation:

- Can explain the runtime path of a bundler feature end to end.
- Can compare designs across real projects without overgeneralizing.
- Can implement a small but real subsystem.
- Can debug performance, correctness, and integration issues.

### Phase 2: 2028-2030, Infrastructure Engineer

Goal:

> Move from tooling usage and feature implementation to infrastructure-level system design.

Possible work directions:

- Compiler.
- Runtime.
- Developer platform.
- AI infrastructure.
- Cloud infrastructure.
- Database or storage infrastructure.
- Distributed system infrastructure.

Expected outputs:

- Own a subsystem at work or in open source.
- Build reliable mental models for OS, database, and distributed systems.
- Read and write design docs at infrastructure-team quality.
- Start making architectural decisions that affect other engineers.

Validation:

- Other engineers ask for judgment on system design and tradeoffs.
- Contributions are no longer only feature-level.
- Work is connected to correctness, performance, scalability, or developer productivity.

### Phase 3: 2031-2036, Principal / Research Engineer

Goal:

> Create or lead a system that others depend on.

Possible forms:

- A runtime.
- A compiler or bundler.
- An agent runtime.
- A developer platform.
- An AI infrastructure component.
- A domain-specific language.
- A verification, debugging, or observability system.

Validation:

- Others cite, depend on, or adopt the work.
- The work contains original system design, not just implementation.
- The work changes a workflow or creates a durable abstraction.

## MIT Mode

If a formal graduate route is not immediately available, use "MIT Mode":

> Each year, attack one serious computer science systems area deeply enough to produce artifacts.

Possible sequence:

- 2026: Compiler and build systems.
- 2027: Operating systems and xv6-style labs.
- 2028: Database internals.
- 2029: Distributed systems and Raft.
- 2030: AI runtime and inference systems such as vLLM-style architectures.

Rules:

- One primary direction per year.
- Course notes are not enough.
- Each direction must produce code, notes, benchmarks, or PRs.
- AI can be used as professor, TA, reviewer, and reading partner, but not as a substitute for doing the hard work.

## CS Expert Formation Hypothesis

Hypothesis:

> A practical CS expert is not someone who has watched every famous course. A practical CS expert can explain, build, measure, and change computing systems across several abstraction layers.

For the current path, "CS expert" means building a T-shaped profile:

- Deep vertical: programming systems, build tools, compilers, runtime, developer infrastructure.
- Strong horizontal: operating systems, algorithms, discrete math, databases, distributed systems, AI systems.
- Research habit: read papers, reproduce core ideas, write design notes, and compare real systems.

The goal is not to become equally strong in all CS areas. The goal is to have enough breadth that deep systems work is not blocked by missing foundations.

### Layered Knowledge Map

Use this as the long-term map:

```text
Mathematical foundation
  -> Discrete math, proof, graph theory, probability, asymptotic analysis

Core programming foundation
  -> Data structures, algorithms, programming languages, compilers

Machine foundation
  -> Computer architecture, operating systems, runtime, memory, concurrency

System foundation
  -> Databases, distributed systems, storage, networking, reliability

AI-era foundation
  -> ML systems, inference runtime, agent runtime, developer-tool AI integration

Output layer
  -> Open-source systems, design docs, technical essays, public talks, production-quality tools
```

### Recommended Order

Given the current Feopack/tooling background and the fact that OS basics are already being studied, use this order:

1. Operating systems first.
   - Reason: OS knowledge makes runtime, process, file system, concurrency, memory, and performance less mysterious.
   - Practical anchor: MIT 6.1810-style xv6 labs and OS notes.

2. Algorithms and discrete math in parallel, but pull-based.
   - Reason: graph, invariants, complexity, and proof will show up in module graph, invalidation, scheduling, caching, and distributed systems.
   - Practical anchor: MIT 6.1200 and 6.006 topics selected by project needs.

3. Compiler/build systems as the main vertical.
   - Reason: this is the current strongest continuity from Feopack and frontend infrastructure.
   - Practical anchor: Feopack, Rspack, SWC/Oxc, source maps, incremental build, plugin systems.

4. Databases and distributed systems after OS basics.
   - Reason: they require stronger mental models for concurrency, fault tolerance, storage, and consistency.
   - Practical anchor: MiniSQL-style database, Raft, MIT 6.5840-style distributed systems labs and papers.

5. AI systems after the systems base is real.
   - Reason: AI infra becomes much more understandable once runtime, scheduling, memory, networking, and distributed execution are familiar.
   - Practical anchor: vLLM-style inference runtime, agent runtime, tool execution sandbox, local developer-infra integration.

### Study Mode

Use two tracks:

#### Fragment Track

Use small time blocks during workdays for:

- AI conversations about concepts.
- Reading one section of a course note.
- Reviewing one OS or algorithm concept.
- Asking for analogies tied to Feopack, Node, Rust, or browser tooling.

This track is for exposure and repeated contact, not mastery.

#### Focus Track

Use longer blocks for:

- Doing labs.
- Solving problem sets.
- Writing proofs.
- Reading source code end to end.
- Writing public notes or design docs.

This track is where real ability forms.

### Near-Term Course Anchors

Do not attempt all of these at once.

- MIT 6.1810 Operating System Engineering: primary OS anchor.
- MIT 6.1200 Mathematics for Computer Science: proof, graph, recurrence, invariants, counting, probability.
- MIT 6.006 Introduction to Algorithms: algorithmic modeling, data structures, performance analysis.
- MIT 6.5840 Distributed Systems: later-stage systems anchor after OS basics.

### Evidence of Progress

Progress should be measured by artifacts:

- One lab completed.
- One proof written.
- One source-reading note published.
- One design doc reviewed.
- One subsystem implemented.
- One benchmark or measurement produced.
- One paper explained with its assumptions and limitations.

Avoid measuring progress by:

- Hours watched.
- Number of bookmarked courses.
- Number of AI conversations.
- Feeling of understanding without reproduction.

## 2026 Monthly Roadmap

Theme:

> Finish Feopack as the first credible systems artifact, turn it into public technical output, then open the next FeoJS ecosystem direction while using MIT-style study as background training.

Constraint:

> Feopack is not meant to become a full Rspack/Webpack clone. It should preserve the core mechanisms needed for learning and demonstration, then stop before compatibility work consumes the whole year.

Learning policy for 2026:

> MIT-style courses will not be completed by intention alone. Use workday fragments and AI conversations for concept exposure, but reserve focused blocks for proofs, labs, source reading, and writing. The main learning anchor remains real artifacts.

### 2026-07, Feopack Closure Plan and Gap Audit

Main question:

> What is already done, what must be finished, and what should be explicitly left out?

Focus:

- Finalize the personal direction spec.
- Treat loader pipeline, JS/Rust boundary, and plugin/lifecycle foundations as mostly completed unless new evidence says otherwise.
- Audit the remaining Feopack gaps: module graph, incremental rebuild thinking, watch mode, source map, tree shaking, code splitting, persistent cache.
- Define the smallest "Feopack complete" boundary.
- Start a lightweight source-reading log.

Minimum outputs:

- This spec exists and has a decision log.
- One Feopack closure checklist.
- One architecture note that explains what Feopack implements and what it intentionally does not implement.
- One ranked list of remaining mechanisms: must finish, nice to study, explicitly out of scope.

Review metrics:

- Can explain why Feopack is a learning artifact instead of a compatibility clone.
- Can name the final 2-3 mechanisms worth implementing before stopping.
- Can defend what is being cut.

### 2026-08, Feopack Finalization

Main question:

> Can Feopack reach a clean, demonstrable end state this month?

Focus:

- Finish the remaining core mechanism, likely module graph plus incremental rebuild design or a minimal implementation.
- Keep plugin/lifecycle work scoped: compiler hooks and basic JS plugin compatibility are enough unless a missing piece blocks the demo.
- Avoid implementing full Module Federation, production resolver parity, full Webpack/Rspack compatibility, or a complete feature matrix.
- Stabilize examples, tests, and README-level explanation.

Minimum outputs:

- One "Feopack final architecture" note.
- One final feature checklist with completed and intentionally skipped items.
- A demo path that can show loader, plugin/lifecycle, module graph, and build output.
- If feasible, one minimal incremental rebuild or watch-mode experiment.

Review metrics:

- Can show Feopack to another engineer in 10 minutes.
- Can explain module graph and invalidation boundaries without hand-waving.
- Can stop adding compatibility features without feeling the project is unfinished.

### 2026-09, Public Writing Sprint

Main question:

> Can the Feopack work become external credibility instead of private learning only?

Focus:

- Convert Feopack work into multiple public technical articles.
- Prefer source-grounded writing over general opinion.
- Show tradeoffs: what was copied, simplified, skipped, or learned from Rspack/Webpack.
- Use articles to expose gaps in understanding and feed the next project.

Minimum outputs:

- 3-5 public or publish-ready articles.
- One article about loader pipeline and JS/Rust boundary.
- One article about plugin/lifecycle design.
- One article about module graph and incremental rebuild thinking.
- One retrospective: "What building a mini bundler actually taught me".

Review metrics:

- Articles contain concrete code paths, diagrams, and tradeoffs.
- At least one article is strong enough to share with engineers outside the current context.
- Writing reveals the next direction rather than only summarizing the past.

### 2026-10, FeoJS Ecosystem Direction

Main question:

> What is the next ecosystem project after Feopack, and why should it exist?

Focus:

- Define the FeoJS ecosystem thesis.
- Choose the next project by leverage, not novelty.
- Candidate directions: developer tooling diagnostics, AI-assisted UI repair, build visualization, plugin playground, source-map explorer, dependency graph inspector, or agent-friendly local dev infrastructure.
- Write a spec before implementation.

Minimum outputs:

- One FeoJS ecosystem strategy note.
- One project spec for the next pit.
- One prototype plan with explicit non-goals.

Review metrics:

- The next project reuses Feopack/tooling assets instead of starting from zero.
- The problem is specific enough to build in 4-8 weeks.
- The project strengthens the Programming Systems direction.

### 2026-11, Next Project Prototype

Main question:

> Can the new FeoJS direction become a concrete prototype rather than another idea?

Focus:

- Build the smallest useful version of the chosen FeoJS project.
- Keep the prototype inspectable and engineer-facing.
- Add instrumentation, visualization, or workflow proof where relevant.
- Continue MIT-style learning in fragments, tied to implementation blockers.

Minimum outputs:

- One working prototype.
- One implementation note.
- One list of technical bottlenecks discovered during the prototype.

Review metrics:

- The prototype can be shown in a short demo.
- The prototype teaches a new systems concept.
- The next iteration is obvious from real usage, not imagination.

### 2026-12, Year-End Consolidation and 2027 Choice

Main question:

> What did Feopack and FeoJS prove, and what should 2027 attack deeply?

Focus:

- Consolidate Feopack, public articles, and the new FeoJS prototype.
- Decide whether 2027 should focus on operating systems, compiler depth, distributed systems, or AI runtime.
- Convert scattered AI-assisted learning into a visible reading/lab plan.
- Update this spec with evidence rather than feelings.

Minimum outputs:

- One year-end review note.
- One portfolio index that links Feopack, articles, and the new prototype.
- One 2027 Q1 roadmap.

Review metrics:

- Can point to code, notes, or PRs instead of only saying "I studied".
- Can explain why Feopack ended where it did.
- Can justify the 2027 focus based on evidence from implementation and writing.

## Graduate School Decision

Graduate school can be valuable, but only under the right framing.

Good reasons to pursue it:

- Build research ability.
- Learn how to propose problems, not only finish assigned tasks.
- Enter a stronger international technical network.
- Move closer to systems, programming languages, distributed systems, compilers, or AI engineering.
- Create a credible external signal that complements work and open-source output.

Weak reasons:

- Pure degree anxiety.
- Escaping current work without a technical direction.
- Assuming any master's degree automatically improves career trajectory.
- Trying to reset directly into frontier model algorithm research without matching preparation.

Preferred directions:

- Systems.
- Programming languages.
- Compilers.
- Distributed systems.
- Software engineering research.
- AI engineering.
- ML systems.
- Human-AI developer tools.

## Career Transition Policy

Do not frame the move as:

```text
Frontend -> AI Algorithm Engineer
```

Frame it as:

```text
Frontend Architecture -> Developer Infrastructure -> Programming Systems -> AI Infrastructure
```

This preserves existing experience while increasing scarcity.

The practical job targets are:

- Frontend Infrastructure Engineer.
- Build Tools Engineer.
- Compiler Engineer.
- Developer Experience / Developer Platform Engineer.
- Web Platform Engineer.
- AI Engineering Platform Engineer.
- AI-native Developer Tooling Engineer.
- Runtime / Systems Engineer.

## Learning Policy

Use AI to compress feedback loops, not to skip formation of ability.

Good AI usage:

- Ask for conceptual explanations tied to current engineering context.
- Ask for Socratic hints instead of direct answers.
- Ask it to review proofs, design docs, and code.
- Ask it to compare one source project against another.
- Ask it to generate practice problems and counterexamples.
- Ask it to summarize a paper, then challenge the summary with the original text.

Bad AI usage:

- Letting it solve labs without personal struggle.
- Mistaking "I understand the explanation" for "I can reproduce the reasoning".
- Outsourcing all writing, proof, and design judgment.
- Accumulating notes without implementation.

## Review Cadence

Review this document every quarter.

Each review should answer:

- What did I build?
- What did I read?
- What did I write?
- What did I contribute publicly?
- What became easier than before?
- What still feels fake or hand-wavy?
- Which assumption changed?
- Should the next quarter narrow, broaden, or pivot?

## Decision Log

### 2026-07-13

Current decision:

> Do not chase an AI-hotspot career reset. Move downward from frontend architecture into programming systems and AI-native developer infrastructure.

Reasoning:

- The current interest profile is systems-oriented.
- Existing frontend and tooling experience is reusable.
- Pure model algorithm research has a higher entry barrier and lower expected value from the current starting point.
- AI increases the importance of reliable tooling, runtime, compiler, workflow, and infrastructure layers.

Next checkpoint:

> Revisit after one quarter of focused compiler/build-system work.

Evidence to collect:

- Feopack/Rspack source-reading notes.
- One concrete subsystem implementation.
- One public technical essay or design doc.
- A list of missing foundations exposed during implementation.
